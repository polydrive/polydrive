package fr.dopolytech.polydrive

import grpc.File
import persistency.MongoConfig

import akka.event.slf4j.Logger
import org.bson.codecs.configuration.CodecRegistries.{
  fromProviders,
  fromRegistries
}
import org.bson.codecs.configuration.CodecRegistry
import org.mongodb.scala.MongoClient.DEFAULT_CODEC_REGISTRY
import org.mongodb.scala.bson.ObjectId
import org.mongodb.scala.bson.codecs.Macros._
import org.mongodb.scala.model.Filters.equal
import org.mongodb.scala.model.Updates._
import org.mongodb.scala.result.{InsertOneResult, UpdateResult}
import org.mongodb.scala.{MongoClient, MongoCollection, Observable}

import scala.concurrent.ExecutionContext.Implicits.global
import scala.concurrent.Future

object FileDocument {
  def apply(base_name: String, path: String): FileDocument = {
    FileDocument(new ObjectId(), base_name, path)
  }

  def from(file: File): FileDocument = {
    FileDocument(new ObjectId(), base_name = file.baseName, path = file.path)
  }
}
case class FileDocument(_id: ObjectId, base_name: String, path: String)

class FileRequester(mongoConfig: MongoConfig) {
  // Taken from improvement of actors into concurrency
  // https://www.chrisstucchio.com/blog/2013/actors_vs_futures.html
  val codecRegistry: CodecRegistry =
    fromRegistries(fromProviders(classOf[FileDocument]), DEFAULT_CODEC_REGISTRY)
  private val logger = Logger(getClass.getName)
  private val client: MongoClient = MongoClient(
    s"mongodb://${mongoConfig.host}/?replicaSet=${mongoConfig.replicaSet}"
  )
  private val db =
    client.getDatabase("polydrive").withCodecRegistry(codecRegistry)
  // Name is current collection to anticipate document versioning pattern
  // where we'll have certainly file_revisions collection
  // see: https://www.mongodb.com/blog/post/building-with-patterns-the-document-versioning-pattern
  private val current_coll: MongoCollection[FileDocument] = db
    .getCollection("current_files")

  def create(x: FileDocument): Future[InsertOneResult] = {
    logger.info("Performing a create on {}", x.path)
    current_coll.insertOne(x).toFuture()
  }

  def update(x: FileDocument): Future[Option[UpdateResult]] = {
    logger.info("Performing an update on {}", x.path)
    val updatedFile: Observable[UpdateResult] = current_coll
      .find(equal("path", x.path))
      .first()
      .map(_ => x)
      .flatMap(file =>
        current_coll.updateOne(
          equal("path", x.path),
          combine(set("base_name", file.base_name))
        )
      )

    updatedFile.toFuture().map(_.headOption)
  }

  def findExists(path: String): Future[Boolean] = {
    findLatest(path).map {
      case Some(_) => true
      case None    => false
    }
  }

  def findLatest(path: String): Future[Option[FileDocument]] = {
    current_coll
      .find(equal("path", path))
      .first()
      .headOption()
  }
}
