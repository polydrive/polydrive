package fr.dopolytech.polydrive
import akka.actor.typed.scaladsl.Behaviors
import akka.actor.typed.{ActorRef, Behavior}

object Server {
  def apply(): Behavior[Ping] = Behaviors.receiveMessage {
    case Ping(m, replyTo) =>
      replyTo ! Pong(m)
      Behaviors.same
  }

  case class Ping(message: String, response: ActorRef[Pong])

  case class Pong(message: String)
}
