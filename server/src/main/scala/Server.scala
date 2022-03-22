package fr.dopolytech.polydrive

import file.{FileClient, MinioConfig}
import grpc.FileManagerServiceHandler
import persistency.MongoConfig

import akka.actor.typed.ActorSystem
import akka.actor.typed.scaladsl.Behaviors
import akka.event.slf4j.Logger
import akka.http.scaladsl.Http
import akka.http.scaladsl.model.{HttpRequest, HttpResponse}
import com.typesafe.config.ConfigFactory

import scala.concurrent.duration.DurationInt
import scala.concurrent.{ExecutionContext, Future}
import scala.util.{Failure, Success}

object Server {
  def main(args: Array[String]): Unit = {
    val conf = ConfigFactory
      .parseString("akka.http.server.preview.enable-http2 = on")
      .withFallback(ConfigFactory.defaultApplication())
      .resolve()

    val system = ActorSystem[Nothing](Behaviors.empty, "Server", conf)
    val minioConfig = MinioConfig(
      conf.getString("minio.url"),
      conf.getString("minio.access_key"),
      conf.getString("minio.secret_key"),
      conf.getString("minio.bucket")
    )
    val mongoConfig = MongoConfig(
      conf.getString("mongo.host"),
      conf.getString("mongo.replicaSet")
    )
    new Server(system).run(conf.getInt("grpc.port"), minioConfig, mongoConfig)
  }
}

class Server(system: ActorSystem[_]) {
  private val logger = Logger(getClass.getName)

  def run(
      port: Int,
      minioConfig: MinioConfig,
      mongoConfig: MongoConfig
  ): Future[Http.ServerBinding] = {
    implicit val sys: ActorSystem[_] = system
    implicit val ec: ExecutionContext = system.executionContext

    val fileClient = new FileClient(minioConfig)
    val server: HttpRequest => Future[HttpResponse] =
      FileManagerServiceHandler(
        new FileManagerServiceImpl(system, fileClient, mongoConfig)
      )

    val bound: Future[Http.ServerBinding] = Http(system)
      .newServerAt(interface = "127.0.0.1", port = port)
      .bind(server)
      .map(_.addToCoordinatedShutdown(hardTerminationDeadline = 10.seconds))

    bound.onComplete {
      case Success(binding) =>
        val address = binding.localAddress
        logger.info(
          "gRPC server bound to {}:{}",
          address.getHostString,
          address.getPort
        )
      case Failure(ex) =>
        logger.error(
          "gRPC server failed to bind gRPC endpoint, terminating system, error: {}",
          ex
        )
        system.terminate()
    }

    bound
  }
}
