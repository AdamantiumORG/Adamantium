#include <stdint.h>
#include <stdio.h>

int main(void) {
    int32_t total = 0;
    for (int32_t index = 0; index < 1000000; ++index) {
        total = (total + index) % 1000003;
    }
    printf("%d\n", total);
    return 0;
}
