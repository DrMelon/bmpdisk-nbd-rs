#!/bin/bash
printf "BMPDisk All-In-One Disconnect Disk Script\n"
printf "=========================================\n"

printf "\t* Unmounting disk...\n"
sudo umount /dev/nbd0
printf "\t* Disconnecting nbd-client...\n"
sudo nbd-client -d /dev/nbd0
printf "\t* Killing nbdkit...\n"
sudo killall nbdkit
printf "\t* Done! BMPDisk disconnected.\n"
