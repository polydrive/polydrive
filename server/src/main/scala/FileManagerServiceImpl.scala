package fr.dopolytech.polydrive

import file.FileClient
import grpc._
import persistency.MongoConfig

import akka.NotUsed
import akka.actor.typed.ActorSystem
import akka.event.slf4j.Logger
import akka.grpc.GrpcServiceException
import akka.stream.scaladsl
import akka.stream.scaladsl.{BroadcastHub, Flow, Keep, MergeHub, Sink, Source}
import com.google.protobuf.empty.Empty
import io.grpc.Status
import io.minio.http.Method

import scala.concurrent.ExecutionContext.Implicits.global
import scala.concurrent.duration.DurationInt
import scala.concurrent.{Await, Future}

class FileManagerServiceImpl(
    system: ActorSystem[_],
    minioClient: FileClient,
    mongoConfig: MongoConfig
) extends FileManagerService {
  private implicit val sys: ActorSystem[_] = system
  // We create a stream that can receive dynamically defined inputs
  // and dynamically defined outputs
  // See more: https://doc.akka.io/docs/akka/current/stream/stream-dynamic.html
  // TODO: add a kill switch
  val (
    inboundHub: Sink[File, NotUsed],
    outboundHub: Source[Notification, NotUsed]
  ) =
    MergeHub
      .source[File]
      .map(file => Notification(Option[File](file)))
      .toMat(BroadcastHub.sink[Notification])(Keep.both)
      // might want to add runWith(Sink.ignore) to not notify
      // when no client available
      .run()
  // We create a flow (sink+source, see definitions) to
  // process, with a backpressure defined
  val busFlow: Flow[File, Notification, NotUsed] =
    Flow
      .fromSinkAndSource(inboundHub, outboundHub)
      .backpressureTimeout(3.seconds)
  private val logger = Logger(getClass.getName)
  private val fileRequester: FileRequester = new FileRequester(mongoConfig)

  override def fileEvent(in: FileEventRequest): Future[FileResponse] = {
    logger.info(
      "[{}] Received file event from {}",
      in.eventType,
      in.getClientName.hostName
    )
    val file = in.getFile
    val file_doc = FileDocument.from(file)

    in.eventType match {
      case FileEventType.CREATE => {
        fileRequester.findExists(file.path).map {
          case true  => fileRequester.update(file_doc)
          case false => fileRequester.create(file_doc)
        }
      }
      case FileEventType.UPDATE => {
        fileRequester.update(file_doc)
      }
      case FileEventType.UNKNOWN => {
        logger.error("Could not identity validate fileEvent type")
        new GrpcServiceException(
          Status.INVALID_ARGUMENT.withDescription("Invalid event type UNKNOWN")
        )
      }
      case FileEventType.DELETE => {
        fileRequester.delete(file_doc)
      }
      case _ => {
        new GrpcServiceException(
          Status.INVALID_ARGUMENT.withDescription(
            "Couldn't understand event type"
          )
        )
      }
    }
    Source.single(in.getFile).viaMat(busFlow)(Keep.right).run()

    val link = minioClient.getPresignedUrl(file.path, Method.PUT)
    Future.successful(
      FileResponse(link)
    )
  }

  override def subscribeNotification(
      in: Empty
  ): scaladsl.Source[Notification, NotUsed] = {
    logger.info("Received a subscribe notification")
    outboundHub
  }

  /** This route will fetch every file currently available in the sync directory.
    * It will answer every file available
    */
  override def indexRequest(in: Empty): Future[IndexRequestResponse] = {
    Future.successful(IndexRequestResponse())
  }

  /** This rpc route allows a client to request to download a single file from
    * the sync directory. It will answer the file metadata and the link where
    * to download the file.
    */
  override def file(in: FileRequest): Future[FileResponse] = {
    logger.info(
      s"a client requested a file. checking if file exists. path=${in.path}"
    )

    // Check if file exists in DB
    val fileRequest = fileRequester.findLatest(in.path) map { file =>
      file.getOrElse(
        throw new GrpcServiceException(
          Status.NOT_FOUND.withDescription("File not found in database")
        )
      )
    }
    val file = Await.result(fileRequest, 10.seconds)

    // Check if file exists in object storage
    if (!minioClient.pathExists(in.path)) {
      throw new GrpcServiceException(
        Status.NOT_FOUND.withDescription("File not found in object storage")
      )
    }

    logger.info(s"generating download link for object=${in.path}")
    val downloadLink = minioClient.getPresignedUrl(in.path, Method.GET)

    Future.successful(
      FileResponse(downloadLink, Some(File(file.base_name, file.path)))
    )
  }
}
