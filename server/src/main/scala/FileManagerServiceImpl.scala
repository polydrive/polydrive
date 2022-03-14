package fr.dopolytech.polydrive

import akka.actor.typed.ActorSystem
import fr.dopolytech.polydrive.grpc.{
  FileManagerService,
  FileRequest,
  FileResponse
}

import scala.concurrent.Future

class FileManagerServiceImpl(system: ActorSystem[_])
    extends FileManagerService {
  private implicit val sys: ActorSystem[_] = system

  override def fileEvent(in: FileRequest): Future[FileResponse] = {
    Future.successful(FileResponse())
  }
}
