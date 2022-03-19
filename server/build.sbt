name := "server"

version := "0.1"

scalaVersion := "2.13.8"

idePackagePrefix := Some("fr.dopolytech.polydrive")

val AkkaVersion = "2.6.18"
val minioVersion = "8.3.7"
val commonsioVersion = "20030203.000550"

libraryDependencies ++= {
  Seq(
    "io.minio" % "minio" % minioVersion,
    "commons-io" % "commons-io" % commonsioVersion,
    "com.typesafe.akka" %% "akka-actor-testkit-typed" % AkkaVersion % Test,
    "com.typesafe.akka" %% "akka-actor-typed" % AkkaVersion,
    "com.typesafe.akka" %% "akka-stream-typed" % AkkaVersion,
    "com.typesafe.akka" %% "akka-discovery" % AkkaVersion,
    "ch.qos.logback" % "logback-classic" % "1.2.11",
    "com.typesafe.scala-logging" %% "scala-logging" % "3.9.4"
  )

}

enablePlugins(AkkaGrpcPlugin)
