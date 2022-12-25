#include "sys/wait.h"
#include "stdio.h"
#include "unistd.h"

int main()
{
    int pid = fork();
    
    sleep(20000);
    return 0;
}
