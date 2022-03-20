#!/bin/bash

# Initiate the replica set into one of mongo instance
echo "Starting replica set initialization"
until mongo --host mongo-rs-1:27017 --eval "print(\"Waited for connection\")"
do
    sleep 2
done
echo "Connection finished"
echo "Creating replica set"
mongo --host mongo-rs-1:27017 <<EOF
    rs.initiate({
        _id : 'polydrive-rs',
        members: [
            { _id : 0, host : "mongo-rs-1:27017" },
            { _id : 1, host : "mongo-rs-2:27017" },
            { _id : 2, host : "mongo-rs-3:27017", arbiterOnly: true }
        ]
      }
    )
EOF
echo "Replica Set created."
