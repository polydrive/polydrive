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
import org.mongodb.scala.model.Sorts._
import org.mongodb.scala.bson.codecs.Macros._
import org.mongodb.scala.model.{Accumulators, Aggregates}
import org.mongodb.scala.model.Filters.equal
import org.mongodb.scala.model.Updates._
import org.mongodb.scala.result.{InsertOneResult, UpdateResult}
import org.mongodb.scala.{MongoClient, MongoCollection, Observable}

import scala.concurrent.ExecutionContext.Implicits.global
import scala.concurrent.Future
import scala.concurrent._
import scala.concurrent.duration._

object FileDocument {
  def apply(
      base_name: String,
      path: String,
      version: Option[Int]
  ): FileDocument = {
    FileDocument(new ObjectId().toString, base_name, path, None, false)
  }

  def from(file: File): FileDocument = {
    FileDocument(
      new ObjectId().toString,
      base_name = file.baseName,
      path = file.path,
      None,
      false
    )
  }
}
case class FileDocument(
    _id: String,
    base_name: String,
    path: String,
    var version: Option[Int],
    var deleted: Boolean
)

class FileRequester(mongoConfig: MongoConfig) {
  // Taken from improvement of actors into concurrency
  // https://www.chrisstucchio.com/blog/2013/actors_vs_futures.html
  val codecRegistry: CodecRegistry =
    fromRegistries(fromProviders(classOf[FileDocument]), DEFAULT_CODEC_REGISTRY)
  private val logger = Logger(getClass.getName)
  val DEFAULT_VERSION: Int = 1
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
    x.version = Some(DEFAULT_VERSION)
    current_coll.insertOne(x).toFuture()
  }

  //update is a new version of the file
  def update(x: FileDocument): Future[InsertOneResult] = {
    logger.info("Performing an update on {}", x.path)
    val latestVersion = findLatestVersion(x.path)
    x.version = Some(latestVersion.get + 1)
    current_coll.insertOne(x).toFuture()
  }

  //delete is a new version of the file with deleted boolean set to True
  def delete(x: FileDocument): Future[InsertOneResult] = {
    logger.info("Performing a delete on {}", x.path)
    val latestVersion = findLatestVersion(x.path)
    x.version = Some(latestVersion.get + 1)
    x.deleted = true
    current_coll.insertOne(x).toFuture()
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
      .sort(orderBy(descending("version")))
      .first()
      .headOption()
  }

  def findLatestVersion(path: String): Option[Int] = {
    var latestVersion: Option[Int] = Some(0)
    val fileRequest = findLatest(path) map { file =>
      file.getOrElse(
        return latestVersion
      )
    }
    val file = Await.result(fileRequest, 10.seconds)
    file.version
  }

  /** Find all files on the DB, and return only their latest versions
    * @return
    */
  def findAll(): Future[Seq[FileDocument]] = {
    current_coll
      .aggregate(
        Seq(
          Aggregates.group(
            "$base_name",
            Accumulators.max("version", "$version"),
            Accumulators.first("path", "$path"),
            Accumulators.first("base_name", "$base_name"),
            Accumulators.first("deleted", "$deleted")
          )
        )
      )
      .toFuture()
  }
}
