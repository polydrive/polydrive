package fr.dopolytech.polydrive
package file

import akka.event.slf4j.Logger
import io.minio.http.Method
import io.minio.{
  BucketExistsArgs,
  GetPresignedObjectUrlArgs,
  MinioClient,
  StatObjectArgs
}

import java.util.concurrent.TimeUnit

class FileClient(minioConfig: MinioConfig) {

  /** The minio client which will be used to perform
    * requests against MinIO instance
    */
  val client: MinioClient = MinioClient
    .builder()
    .endpoint(minioConfig.url)
    .credentials(
      minioConfig.accessKey,
      minioConfig.secretKey
    )
    .build()

  /** A boolean to control if the buckets is created or not.
    */
  val isBucketCreated: Boolean =
    client.bucketExists(
      BucketExistsArgs
        .builder()
        .bucket(minioConfig.bucket)
        .build()
    )
  private val logger = Logger(getClass.getName)

  def pathExists(path: String): Boolean = {
    val args = StatObjectArgs
      .builder()
      .bucket(minioConfig.bucket)
      .`object`(path)
      .build();

    try {
      client.statObject(args)
      true
    } catch {
      case e: io.minio.errors.ErrorResponseException => {
        logger.error(s"${e.getMessage} = $path")
        false
      }
    }
  }

  /** Generate an upload URL for a file
    *
    * @return
    */
  def getPresignedUrl(path: String, method: Method = Method.PUT): String = {
    if (!isBucketCreated) throw new RuntimeException("bucket not created")

    val args = GetPresignedObjectUrlArgs
      .builder()
      .method(method)
      .bucket(minioConfig.bucket)
      .expiry(15, TimeUnit.MINUTES)
      .`object`(path)
      .build()

    client.getPresignedObjectUrl(args)
  }
}
