#include "sys/wait.h"
#include "stdio.h"
#include "unistd.h"

int main(int argc, char *argv[])
{
    int pid = fork();

    int status = 0;

    if (pid == 0)
    {
        fork();
        char *newArgv[argc];

        for (int i = 1; i < argc; i++)
        {
            newArgv[i - 1]=argv[i];
        }
        newArgv[argc - 1] = NULL;
        
        execvp(argv[1],newArgv);
    }
    else
    {
        printf("parent %d\n", getpid());
        waitpid(pid, &status, 0);

        if(status != 0)
        {
            printf("Failed, exit code = %d\n", status);
        }
        else
        {
            printf("Succes!\n");
        }
    }  

    return 0; 
}
