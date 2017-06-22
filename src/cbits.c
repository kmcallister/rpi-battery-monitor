//  How to access GPIO registers from C-code on the Raspberry-Pi
//  Example program
//  15-January-2012
//  Dom and Gert
//  Revised: 15-Feb-2013
//
//  Munged into a very simple library by Keegan McAllister 20-Jun-2017
 
#define BCM2708_PERI_BASE        0x3F000000
#define GPIO_BASE                (BCM2708_PERI_BASE + 0x200000) /* GPIO controller */
 
#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <sys/mman.h>
#include <unistd.h>
 
#define PAGE_SIZE (4*1024)
#define BLOCK_SIZE (4*1024)
 
static int  mem_fd;
static void *gpio_map;
 
// I/O access
static volatile unsigned *gpio;
 
// GPIO setup macros. Always use INP_GPIO(x) before using OUT_GPIO(x) or SET_GPIO_ALT(x,y)
#define INP_GPIO(g) *(gpio+((g)/10)) &= ~(7<<(((g)%10)*3))
#define OUT_GPIO(g) *(gpio+((g)/10)) |=  (1<<(((g)%10)*3))
#define SET_GPIO_ALT(g,a) *(gpio+(((g)/10))) |= (((a)<=3?(a)+4:(a)==4?3:2)<<(((g)%10)*3))
 
#define GPIO_SET *(gpio+7)  // sets   bits which are 1 ignores bits which are 0
#define GPIO_CLR *(gpio+10) // clears bits which are 1 ignores bits which are 0
 
#define GET_GPIO(g) (*(gpio+13)&(1<<g)) // 0 if LOW, (1<<g) if HIGH
 
#define GPIO_PULL *(gpio+37) // Pull up/pull down
#define GPIO_PULLCLK0 *(gpio+38) // Pull up/pull down clock

static const int pin = 4;

// Set up a memory region to access GPIO
void gpio_init() {
    if ((mem_fd = open("/dev/mem", O_RDWR|O_SYNC)) < 0) {
        printf("can't open /dev/mem \n");
        exit(-1);
    }

    gpio_map = mmap(
        NULL,                 // Any adddress in our space will do
        BLOCK_SIZE,           // Map length
        PROT_READ|PROT_WRITE, //  Enable reading & writting to mapped memory
        MAP_SHARED,           // Shared with other processes
        mem_fd,               // File to map
        GPIO_BASE);           // Offset to GPIO peripheral

    close(mem_fd);

    if (gpio_map == MAP_FAILED) {
        printf("mmap error %d\n", (int) gpio_map);
        exit(-1);
    }

    gpio = (volatile unsigned *)gpio_map;

    // Switch GPIO4 to output mode
    INP_GPIO(pin); // must use INP_GPIO before we can use OUT_GPIO
    OUT_GPIO(pin);
}

int gpio_read() {
    return GET_GPIO(pin) != 0;
}
