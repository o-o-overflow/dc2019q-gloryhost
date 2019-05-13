#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <x86intrin.h>

unsigned int _data_size = 16;
uint8_t _data2[64];
uint8_t _data3[16] = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16};
uint8_t _data4[64];
uint8_t _data5[256 * 512];
char _data6[64] = {0,};
uint8_t _temp = 0;

void check_data(size_t x) {
    if (x < _data_size) {
        _temp &= _data5[_data3[x] * 512];
    }
}

void init_data() {
    for (int i = 0; i < (int) sizeof(_data5); i++) {
        _data5[i] = 1;
    }
}
