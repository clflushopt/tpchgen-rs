#!/bin/bash
#
# Runs the dbgen command in docker to generate tbl data at various scales,

set -x
set -e

LOGFILE=tbl_dbgen_docker.txt

echo "Timings" >> $LOGFILE
echo "***********Timings**********" >> $LOGFILE
date >> $LOGFILE
uname -a >> $LOGFILE

# Fetch the image locally


mkdir -p out-dbgen_docker
SCALE_FACTORS="1 10 100"
for sf in $SCALE_FACTORS ; do
    echo "SF=$sf" >> $LOGFILE
    /usr/bin/time -a -o $LOGFILE docker run -v "/tmp:/data" --rm ghcr.io/scalytics/tpch-docker:main -vf -s 1
done
