name := "server"

version := "0.1"

scalaVersion := "2.13.8"

idePackagePrefix := Some("fr.dopolytech.polydrive")

val AkkaVersion = "2.6.18"
val minioVersion = "8.3.7"
val commonsioVersion = "20030203.000550"

libraryDependencies ++= Seq(
  "com.typesafe.akka" %% "akka-actor-typed" % AkkaVersion,
  "com.typesafe.akka" %% "akka-stream-typed" % AkkaVersion,
  "com.typesafe.akka" %% "akka-discovery" % AkkaVersion,
  "ch.qos.logback" % "logback-classic" % "1.2.11"
)

// MINIO
libraryDependencies ++= Seq(
  "io.minio" % "minio" % minioVersion,
  "commons-io" % "commons-io" % commonsioVersion
)

// TESTS
libraryDependencies ++= Seq(
  "org.scalatest" %% "scalatest" % "3.2.11" % Test,
  "com.typesafe.akka" %% "akka-stream-testkit" % AkkaVersion % Test,
  "com.typesafe.akka" %% "akka-actor-testkit-typed" % AkkaVersion % Test
)

// Database & persistence of data
libraryDependencies ++= Seq(
  "com.lightbend.akka" %% "akka-stream-alpakka-mongodb" % "3.0.4",
  "com.typesafe.akka" %% "akka-stream-typed" % AkkaVersion
)

enablePlugins(AkkaGrpcPlugin)
