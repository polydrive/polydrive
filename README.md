# Polydrive

Polydrive is an experimental open source alternative to Google Drive. It allows users to synchronize their files on multiple devices.

**The project is experimental and should not be used in any production systems.**

Polydrive is written in **Scala** for the file manager, and **Rust** for the client. It makes use of an external database, **MongoDB** to store data about the files, and those files are uploaded to an object storage system, **MinIO**.


