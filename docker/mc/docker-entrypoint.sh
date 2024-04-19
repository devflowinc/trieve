#!/bin/bash
mc config host add myminio http://${S3_URI} ${MINIO_ROOT_USER} ${MINIO_ROOT_PASSWORD};
mc alias set myminio http://${S3_URI} ${MINIO_ROOT_USER} ${MINIO_ROOT_PASSWORD};
mc admin user add myminio ${S3_ACCESS_KEY} ${S3_SECRET_KEY};
mc admin policy attach myminio readwrite --user ${S3_ACCESS_KEY};
mc mb myminio/${S3_BUCKET};
exit $?;
