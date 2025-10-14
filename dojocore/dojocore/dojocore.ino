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

char modes[] = { 0, 0, 0 };

int pinCount = sizeof(pins) / sizeof(pindef);

void setup() {
  Serial.begin(115200);
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
    while (Serial.available() == 0) {} // block until byte
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
    else if (byte == 0x86) {
      Serial.write(0xFF);
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
    }
    else {
      Serial.write(0xFE); // send an error code and get out of here skoob
      return;
    }
  }
}
