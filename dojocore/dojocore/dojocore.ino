// the core of CircuitDojo

struct pindef {
  int physical_pin;
  bool is_analog;
  bool has_pullup;
  const char* identifier;
};

pindef pins[] = {
  { 3, false, true, "Digital 3" },
  { 4, false, true, "Digital 4" },
  { A2, true, false, "Analog 2" }
};

#define DIG_NONE 1024
#define DIG_LOW 1025
#define DIG_HIGH 1026

uint16_t states[] = { // 0-1023 = analog, 1024=none, 1025=digital low, 1026=digital high
  DIG_NONE,
  DIG_NONE,
  DIG_NONE
};

char modes[] = { 0, 0, 0 }; // 0=none, 1=input, 2=output

const int pinCount = sizeof(pins) / sizeof(pindef);

void setup() {
  Serial.begin(115200);
}

uint16_t subsc_wavelength = 0;
long last_update = 0;

void doPinUpdates() {
  for (int i = 0; i < pinCount; i ++) {
    if (modes[i] != 1) {
      continue;
    }
    if (pins[i].is_analog) {
  
    }
    else {
      int val = digitalRead(pins[i].physical_pin);
      int state = val ? DIG_HIGH : DIG_LOW;
      if (states[i] != state) {
        states[i] = state;
        Serial.write(i | (val ? 0x40 : 0));
      }
    }
  }
}

void loop() {
  while (Serial.available() == 0) {} // block until byte
  int byte = Serial.read();
  if (byte == 0xFF) {
    Serial.write(0xFF);
  }
  else {
    return;
  }

  while (true) {
    // past this point the handshake is complete! let's do some normal operation tasks:
    while (Serial.available() == 0) {
      if (subsc_wavelength != 0) {
        if (millis() - last_update > subsc_wavelength) {
          last_update = millis();
          doPinUpdates();
        }
      }
    }
    byte = Serial.read();
    if ((~byte) & 0x80) { // if the high bit is unset
      int pindex = byte & 0b00111111;
      if (pins[pindex].is_analog) {
        // TODO
      }
      else {
        digitalWrite(pins[pindex].physical_pin, (byte & 0x40) ? HIGH : LOW);
      }
    }
    else if (byte == 0x80) {
      uint8_t capabilities[] = { 0xFF, 0x80, 0x10, 0x00 };
      Serial.write(capabilities, 4);
      for (int i = 0; i < pinCount; i ++) {
        Serial.write(0x81);
        Serial.write(i | (pins[i].is_analog ? 0x80 : 0x00) | (pins[i].has_pullup ? 0x40 : 0x00));
        Serial.write(pins[i].identifier);
        Serial.write(0);
      }
      Serial.write(0x82);
      Serial.write("Arduino UNO R3 running CircuitDojo");
      Serial.write(0);
    }
    else if (byte == 0x81) {
      while (Serial.available() == 0) {} // block until byte
      int pindex = Serial.read();
      if (pindex == -1) {
        Serial.write(0xFE);
      }
      else {
        Serial.write(0xFF);
        pinMode(pins[pindex].physical_pin, INPUT);
        modes[pindex] = 1;
      }
    }
    else if (byte == 0x82) {
      while (Serial.available() == 0) {} // block until byte
      int pindex = Serial.read();
      if (pindex == -1) {
        Serial.write(0xFE);
      }
      else {
        Serial.write(0xFF);
        pinMode(pins[pindex].physical_pin, OUTPUT);
        modes[pindex] = 2;
      }
    }
    else if (byte == 0x84) { // subscribe to updates
      while (Serial.available() < 2) {} // block until bytes
      subsc_wavelength = Serial.read();
      subsc_wavelength |= Serial.read() << 8;
      Serial.write(0xFF);
    }
    else if (byte == 0x86) { // write all pin values, then ACK
      for (int i = 0; i < pinCount; i ++) {
        if (modes[i] == 1) {
          if (pins[i].is_analog) {
            // TODO
          }
          else {
            Serial.write(i | (digitalRead(pins[i].physical_pin) ? 0x40 : 0));
          }
        }
      }
      Serial.write(0xFF);
    }
    else {
      Serial.write(0xFE); // send an error code and get out of here skoob
      return;
    }
  }
}
