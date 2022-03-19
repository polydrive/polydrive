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

class FileManagerServiceImpl(system: ActorSystem[_], minioClient: FileClient)
    extends FileManagerService {
  private val logger = Logger(getClass.getName)
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

  override def fileEvent(in: FileRequest): Future[FileResponse] = {
    logger.info(
      "[{}] Received file event from {}",
      in.eventType,
      in.getClientName.hostName
    )
    val file = in.getFile
    val link = minioClient.getPresignedUrl(file.baseName, Method.PUT)

    Source.single(in.getFile).viaMat(busFlow)(Keep.right).run()
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
}
