Use this with nbdkit on Linux.

Build with Cargo: `cargo build --release`

BMP stored in `/var/tmp/bmpdisk.bmp`

`nbdkit ./target/release/libbmpdisk.so`  
Runs nbdkit.   

Options:  
`filename=/var/tmp/bmpdisk.bmp   Target file location. Makes a new file if it doesn't already exist.`   

`dimensions=1024  Sets the dimensions of the image when creating a new file.`  

`no-split-channels=true   If set, does not split disk use across R, G, B channels.`    

## Loading with nbd and mounting disk  
`sudo modprobe nbd`  
Load nbd driver (might need to do this before nbdkit.)  

`sudo nbd-client -b 512 localhost 10809 /dev/nbd0`  
Connect disk to system in loopback file.  
To disconnect: `sudo nbd-client -d /dev/nbd0`  

`sudo gdisk /dev/nbd0`  
Create empty partition table.  

`mkfs -t ext4 /dev/nbd0`  
Create filesystem on disk.  

`sudo mount /dev/nbd0 /mnt/bmpdisk`  
Mount disk.  

`sudo chmod -R 755 /mnt/bmpdisk && sudo chown -R <username> /mnt/bmpdisk`  
Take ownership of disk.  
