package fr.dopolytech.polydrive
package file

case class MinioConfig(
    url: String,
    accessKey: String,
    secretKey: String,
    bucket: String
)
