package fr.dopolytech.polydrive

import file.FileClient
import grpc.{FileManagerService, FileRequest, FileResponse}

import akka.actor.typed.ActorSystem
import io.minio.http.Method

import scala.concurrent.Future

class FileManagerServiceImpl(system: ActorSystem[_])
    extends FileManagerService {
  private implicit val sys: ActorSystem[_] = system

  /** Upload client
    */
  private val fileClient = new FileClient()

  override def fileEvent(in: FileRequest): Future[FileResponse] = {
    val file = in.getFile
    val link = fileClient.getPresignedUrl(file.baseName, Method.PUT)

    Future.successful(
      FileResponse(link)
    )
  }
}
