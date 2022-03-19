package fr.dopolytech.polydrive
package file

import com.typesafe.config.Config
import io.minio.http.Method
import io.minio.{BucketExistsArgs, GetPresignedObjectUrlArgs, MinioClient}

import java.util.concurrent.TimeUnit

class FileClient(config: Config) {

  /** The minio client which will be used to perform
    * requests against MinIO instance
    */
  val client: MinioClient = MinioClient
    .builder()
    .endpoint(config.getString("minio.url"))
    .credentials(
      config.getString("minio.access_key"),
      config.getString("minio.secret_key")
    )
    .build()

  /** The bucket name
    */
  val bucket = config.getString("minio.bucket")

  /** A boolean to control if the buckets is created or not.
    */
  val isBucketCreated: Boolean =
    client.bucketExists(
      BucketExistsArgs
        .builder()
        .bucket(bucket)
        .build()
    )

  /** Generate an upload URL for a file
    *
    * @return
    */
  def getPresignedUrl(fileName: String, method: Method = Method.PUT): String = {
    if (!isBucketCreated) throw new RuntimeException("bucket not created")

    val args = GetPresignedObjectUrlArgs
      .builder()
      .method(method)
      .bucket(bucket)
      .expiry(15, TimeUnit.MINUTES)
      .`object`(fileName)
      .build()

    client.getPresignedObjectUrl(args)
  }
}
