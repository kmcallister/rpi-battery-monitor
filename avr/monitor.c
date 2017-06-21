#define F_CPU 1200000

#include <stdint.h>
#include <avr/io.h>
#include <avr/sleep.h>
#include <util/delay.h>

#define OUT (1 << DDB1)
#define MANCHESTER_DELAY 500

inline void out_high() {
    PORTB |= OUT;
}

inline void out_low() {
    PORTB &= ~OUT;
}

void manchester_out(uint8_t bit) {
    if (bit) {
        out_high();
        _delay_us(MANCHESTER_DELAY);
        out_low();
        _delay_us(MANCHESTER_DELAY);
    } else {
        out_low();
        _delay_us(MANCHESTER_DELAY);
        out_high();
        _delay_us(MANCHESTER_DELAY);
    }
}

int main() {
    ADMUX = (1 << MUX1);  // ADC2 (PB4)

    ADCSRA = (1 << ADPS2) // ADC freq 9.6 MHz / 8 (CKDIV8) / 16 (ADPS) = 75 kHz
           | (1 << ADEN); // enable ADC!

    // Set as output.
    DDRB = OUT;

    out_low();

    for (;;) {
        // Start a conversion and wait for it to finish.
        ADCSRA |= (1 << ADSC);
        while (ADCSRA & (1 << ADSC));

        // Read the ADC result.
        uint16_t reading = ADCL;
        reading |= ((uint16_t) ADCH) << 8;

        // Add a recognizable signature.
        reading |= 0xB400;

        // Ouput the value in Manchester code with a leading '1' bit preamble.
        manchester_out(1);
        for (uint8_t i=0; i<16; i++) {
            manchester_out(reading & 1);
            reading >>= 1;
        }

        // Restore the data line to the 'low' state.
        out_low();

        _delay_ms(100);
    }
}
