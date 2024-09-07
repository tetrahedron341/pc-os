/* ENTRY(_start) */

SECTIONS
{
    . = 0x10000000;                 /* relocate to address 0x1000_0000 */

    .text ALIGN(4):
    {
        *(.text)           /* include all other .text sections */
    }

    .data ALIGN(4):
    {
        *(.data)
    }

    .rodata ALIGN(4):
    {
        *(.rodata*)
    }
}