#!/bin/bash

printf "BMPDisk All-In-One Setup & Init Script\n"
printf "=========================================\n"


printf "\t* Switching directory to $(pwd)/../\n"
cd ..
printf "\t* Load kernel module.\n"
sudo modprobe nbd 
printf "\t* Build BMPDisk.\n"
cargo build --release
printf "\t* Run nbdkit bmpdisk plugin, dimensions 2048x2048 (48 MB), in /var/tmp/bmpdisk.bmp\n"
nbdkit ./target/release/libbmpdisk.so dimensions=2048 filename=/var/tmp/bmpdisk.bmp
printf "\t* Start nbd-client loopback.\n"
sudo nbd-client -b 512 localhost 10809 /dev/nbd0
printf "\t* Wait for a moment"
sleep 1 &
wait -n 
printf "."
sleep 1 &
wait -n
printf "."
sleep 1 &
wait -n 
printf ". [OK!]\n"
printf "\t* Make new ext4 filesystem in /dev/nbd0.\n"
sudo mkfs -t ext4 /dev/nbd0 
printf "\t* Make mountpoint and mount, take ownership of new drive.\n"
sudo mkdir -p /mnt/bmpdisk/
sudo mount /dev/nbd0 /mnt/bmpdisk
sudo chown -R $(whoami) /mnt/bmpdisk
echo "Welcome!" > /mnt/bmpdisk/hello.txt
printf "\t* Done!\n"
