#include <stdint.h>

typedef struct {
    uint16_t a;
    uint16_t b;
    uint16_t c;
    uint16_t i;
    uint16_t j;
    uint16_t x;
    uint16_t y;
    uint16_t z;
    uint16_t pc;
    uint16_t ia;
    uint16_t sp;
    uint16_t ex;
} CRegisters;

typedef struct Debugger Debugger;

Debugger *dcpu_debugger_new();
uint16_t *dcpu_debugger_ram(Debugger*);
CRegisters dcpu_debugger_registers(Debugger*);
void dcpu_debugger_step(Debugger*);
void dcpu_debugger_continue(Debugger*);
void dcpu_debugger_free(Debugger*);
