#!/usr/bin/env node

import { readFileSync, existsSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const PRIVACY_URL = process.env.PRIVACY_URL || 'https://example.com/privacy';

let errors = 0;

function checkLength(file, maxLength, label) {
  if (!existsSync(file)) {
    console.error(`❌ Missing: ${file}`);
    errors++;
    return;
  }
  
  const content = readFileSync(file, 'utf-8').trim();
  const length = content.length;
  
  if (length > maxLength) {
    console.error(`❌ ${label} too long: ${length} > ${maxLength} chars`);
    console.error(`   File: ${file}`);
    errors++;
  } else {
    console.log(`✅ ${label}: ${length}/${maxLength} chars`);
  }
}

function validatePrivacyURL() {
  try {
    const url = new URL(PRIVACY_URL);
    if (url.protocol !== 'https:') {
      console.error(`❌ Privacy URL must use HTTPS: ${PRIVACY_URL}`);
      errors++;
    } else {
      console.log(`✅ Privacy URL format: ${PRIVACY_URL}`);
    }
  } catch (e) {
    console.error(`❌ Invalid Privacy URL: ${PRIVACY_URL}`);
    errors++;
  }
}

console.log('🔍 Validating store listing content...\n');

// Google Play Store
console.log('📱 Google Play Store:');
checkLength(join(__dirname, 'play/short.txt'), 80, 'Short description');
checkLength(join(__dirname, 'play/full.txt'), 4000, 'Full description');

// Apple App Store
console.log('\n🍎 Apple App Store:');
checkLength(join(__dirname, 'appstore/promo.txt'), 4000, 'Promotional text');
checkLength(join(__dirname, 'appstore/keywords.txt'), 100, 'Keywords');
checkLength(join(__dirname, 'appstore/subtitle.txt'), 30, 'Subtitle');

// Privacy URL
console.log('\n🔒 Privacy Policy:');
validatePrivacyURL();

// Summary
console.log('\n' + '='.repeat(50));
if (errors === 0) {
  console.log('✅ All checks passed!');
  process.exit(0);
} else {
  console.error(`❌ ${errors} error(s) found`);
  process.exit(1);
}

