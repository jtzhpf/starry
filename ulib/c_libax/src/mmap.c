#include <stddef.h>
#include <stdio.h>
#include <sys/types.h>

// TODO:
void *mmap(void *addr, size_t len, int prot, int flags, int fildes, off_t off)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);

    return NULL;
}

// TODO:
int munmap(void *addr, size_t length)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
void *mremap(void *old_address, size_t old_size, size_t new_size, int flags,
             ... /* void *new_address */)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return NULL;
}
