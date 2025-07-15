#include <stdio.h>

// Array summation benchmark - C version
int array_sum() {
    int sum = 0;
    for (int i = 0; i < 1000000; i++) {
        sum += i;
    }
    return sum;
}

int main() {
    int result = array_sum();
    printf("Array sum result: %d\n", result);
    return 0;
}