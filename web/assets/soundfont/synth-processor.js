// synth-processor.js — AudioWorkletProcessor wrapping the OxiSynth WASM module.
// Receives a compiled WebAssembly.Module via processorOptions, instantiates it
// asynchronously (to avoid mobile sync size limits), and renders audio via process().

// Parse SF2 preset headers (PHDR) from raw SF2 bytes.
// Returns array of { bank, program, name } sorted by bank then program.
// Filters out percussion (bank >= 120) and presets >= 120.
// Decode ASCII bytes to string (TextDecoder is unavailable in AudioWorklet scope).
function asciiDecode(bytes, offset, length) {
  let s = '';
  for (let i = 0; i < length; i++) {
    const c = bytes[offset + i];
    if (c === 0) break;
    s += String.fromCharCode(c);
  }
  return s.trim();
}

function parseSF2Presets(bytes) {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const presets = [];

  // Find the PHDR sub-chunk by walking the RIFF structure.
  // SF2 layout: RIFF('sfbk' LIST('INFO' ...) LIST('sdta' ...) LIST('pdta' phdr ...))
  try {
    let pos = 12; // skip RIFF header (4) + size (4) + 'sfbk' (4)
    const end = bytes.length;

    while (pos < end - 8) {
      const chunkId = asciiDecode(bytes, pos, 4);
      const chunkSize = view.getUint32(pos + 4, true);

      if (chunkId === 'LIST') {
        const listType = asciiDecode(bytes, pos + 8, 4);
        if (listType === 'pdta') {
          // Search for phdr within pdta
          let sub = pos + 12;
          const listEnd = pos + 8 + chunkSize;
          while (sub < listEnd - 8) {
            const subId = asciiDecode(bytes, sub, 4);
            const subSize = view.getUint32(sub + 4, true);
            if (subId === 'phdr') {
              // Each preset header record is 38 bytes
              const recordCount = Math.floor(subSize / 38);
              for (let i = 0; i < recordCount; i++) {
                const rOff = sub + 8 + i * 38;
                const name = asciiDecode(bytes, rOff, 20);
                const program = view.getUint16(rOff + 20, true);
                const bank = view.getUint16(rOff + 22, true);
                // Skip terminator "EOP" and percussion/drum banks
                if (name !== 'EOP' && bank < 120 && program < 120) {
                  presets.push({ bank, program, name });
                }
              }
              break;
            }
            sub += 8 + subSize + (subSize % 2); // pad to even
          }
          break;
        }
      }
      pos += 8 + chunkSize + (chunkSize % 2); // pad to even
    }
  } catch (e) {
    // If parsing fails, return empty — synth still works
  }

  presets.sort((a, b) => a.bank - b.bank || a.program - b.program);
  return presets;
}

class SynthProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super();

    this.wasm = null;
    this.synth = 0;
    this.leftPtr = 0;
    this.rightPtr = 0;
    this.ready = false;

    this.port.onmessage = (e) => this.handleMessage(e.data);

    // Async WASM instantiation — avoids the sync size limit on mobile browsers.
    // processorOptions.wasmModule is a compiled WebAssembly.Module from the main thread.
    const wasmModule = options.processorOptions && options.processorOptions.wasmModule;
    if (wasmModule) {
      this.initWasm(wasmModule);
    } else {
      this.port.postMessage({
        type: 'error',
        message: 'wasmModule missing from processorOptions (type: ' + typeof wasmModule + ')'
      });
    }
  }

  async initWasm(wasmModule) {
    try {
      const instance = await WebAssembly.instantiate(wasmModule, {});
      this.wasm = instance.exports;

      this.synth = this.wasm.synth_new(sampleRate);
      if (this.synth === 0) {
        this.port.postMessage({ type: 'error', message: 'synth_new returned null' });
        return;
      }

      this.leftPtr = this.wasm.alloc(128 * 4);
      this.rightPtr = this.wasm.alloc(128 * 4);

      this.ready = true;
      this.port.postMessage({ type: 'ready' });
    } catch (e) {
      this.port.postMessage({ type: 'error', message: 'WASM instantiation failed: ' + e });
    }
  }

  handleMessage(msg) {
    switch (msg.type) {
      case 'loadSoundFont': {
        if (!this.wasm) break;
        const sf2Bytes = new Uint8Array(msg.data);
        const presets = parseSF2Presets(sf2Bytes);
        const len = sf2Bytes.length;
        const ptr = this.wasm.alloc(len);
        new Uint8Array(this.wasm.memory.buffer, ptr, len).set(sf2Bytes);
        const result = this.wasm.synth_load_soundfont(this.synth, ptr, len);
        this.wasm.dealloc(ptr, len);
        if (result === 0) {
          this.port.postMessage({ type: 'soundFontLoaded', presets });
        } else {
          this.port.postMessage({ type: 'error', message: 'synth_load_soundfont failed' });
        }
        break;
      }
      case 'noteOn':
        if (this.wasm) this.wasm.synth_note_on(this.synth, msg.key, msg.vel);
        break;
      case 'noteOff':
        if (this.wasm) this.wasm.synth_note_off(this.synth, msg.key);
        break;
      case 'pitchBend':
        if (this.wasm) this.wasm.synth_pitch_bend(this.synth, msg.value);
        break;
      case 'selectProgram':
        if (this.wasm) this.wasm.synth_select_program(this.synth, msg.bank, msg.preset);
        break;
      case 'allNotesOff':
        if (this.wasm) this.wasm.synth_all_notes_off(this.synth);
        break;
    }
  }

  process(inputs, outputs, parameters) {
    if (!this.ready || this.synth === 0) {
      return true;
    }

    const output = outputs[0];
    if (!output || output.length === 0) {
      return true;
    }

    const left = output[0];
    const right = output.length > 1 ? output[1] : null;
    const len = left.length;

    // Render samples via WASM
    this.wasm.synth_render(this.synth, this.leftPtr, this.rightPtr, len);

    // Copy rendered samples from WASM memory to output buffers
    left.set(new Float32Array(this.wasm.memory.buffer, this.leftPtr, len));
    if (right) {
      right.set(new Float32Array(this.wasm.memory.buffer, this.rightPtr, len));
    }

    return true; // Keep processor alive
  }
}

registerProcessor('synth-processor', SynthProcessor);
