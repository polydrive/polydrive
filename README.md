# Polydrive

Polydrive is an experimental open source alternative to Google Drive. It allows users to synchronize their files on multiple devices.

**The project is experimental and should not be used in any production systems.**

Polydrive is written in **Scala** for the file manager, and **Rust** for the client. It makes use of an external database, **MongoDB** to store data about the files, and those files are uploaded to an object storage system, **MinIO**.

# Development

Run the `docker-compose.yml` at the root of the project in order to run development services : 

```shell
$ docker-compose up -d
```

> Mongo URL : `mongodb://localhost:27017,localhost:27018/?replicaSet=polydrive-rs`
> 
> Minio access key : polydrive
> 
> Minio secret key : polydrive
> 
> Minio console : http://localhost:9001
> 
> Minio API : http://localhost:9000

Add the following entry to your `/etc/hosts` file :

```text
127.0.0.1 mongo-rs-1 mongo-rs-2 mongo-rs-3
```

In a terminal, run the server : 

```shell
$ cd server && sbt
$ sbt:polydrive> ~run
```

In another terminal, run a client as a daemon: 

```shell
$ cd client
$ cargo run -- --daemon --verbose --watch /tmp
```

