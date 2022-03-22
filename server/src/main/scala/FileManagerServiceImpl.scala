package fr.dopolytech.polydrive

import grpc._
import file.FileClient
import akka.NotUsed
import akka.actor.typed.ActorSystem
import akka.event.slf4j.Logger
import akka.grpc.GrpcServiceException
import akka.stream.scaladsl
import akka.stream.scaladsl.{BroadcastHub, Flow, Keep, MergeHub, Sink, Source}
import com.google.protobuf.empty.Empty
import io.grpc.Status
import io.minio.http.Method

import scala.concurrent.Future
import scala.concurrent.duration.DurationInt
import scala.concurrent.ExecutionContext.Implicits.global
import persistency.MongoConfig

class FileManagerServiceImpl(
    system: ActorSystem[_],
    minioClient: FileClient,
    mongoConfig: MongoConfig
) extends FileManagerService {
  private val logger = Logger(getClass.getName)
  private implicit val sys: ActorSystem[_] = system
  private val fileRequester: FileRequester = new FileRequester(mongoConfig)

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
        logger.error("Trying to delete file but not implemented")
        new GrpcServiceException(
          Status.UNIMPLEMENTED
        )
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

    val link = minioClient.getPresignedUrl(file.baseName, Method.PUT)
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
  override def indexRequest(in: Empty): Future[IndexRequestResponse] = ???

  /** This rpc route allows a client to request to download a single file from
    * the sync directory. It will answer the file metadata and the link where
    * to download the file.
    */
  override def file(in: FileResponse): Future[FileResponse] = ???
}
