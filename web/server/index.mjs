// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

// Main entry point for web server
// Chooses between Docker mode and local mode

const USE_DOCKER = process.env.USE_DOCKER === 'true' || process.env.USE_DOCKER === '1';

if (USE_DOCKER) {
    console.log('Starting in Docker mode...');
    import('./docker-bridge.mjs').then(() => {
        console.log('Docker bridge server started');
    }).catch(err => {
        console.error('Failed to start Docker bridge:', err);
        console.log('Falling back to local mode...');
        import('./server.mjs');
    });
} else {
    console.log('Starting in local mode...');
    import('./server.mjs');
}

