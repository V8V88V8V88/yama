
#include <stdio.h>
#include <stdlib.h>

void download_package(const char* url, const char* destination) {
    printf("Downloading package from %s to %s\n", url, destination);
}

void extract_package(const char* package, const char* destination) {
    printf("Extracting %s to %s\n", package, destination);
}
