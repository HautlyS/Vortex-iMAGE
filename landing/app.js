/**
 * iMAGE Landing Page - Interactive JavaScript
 */

(function() {
    'use strict';

    const $ = s => document.querySelector(s);
    const $$ = s => document.querySelectorAll(s);

    // Cursor glow effect
    const cursorGlow = $('.cursor-glow');
    if (cursorGlow && window.matchMedia('(pointer: fine)').matches) {
        let mouseX = 0, mouseY = 0, glowX = 0, glowY = 0;
        
        document.addEventListener('mousemove', e => {
            mouseX = e.clientX;
            mouseY = e.clientY;
        });

        function animateGlow() {
            glowX += (mouseX - glowX) * 0.1;
            glowY += (mouseY - glowY) * 0.1;
            cursorGlow.style.left = glowX + 'px';
            cursorGlow.style.top = glowY + 'px';
            requestAnimationFrame(animateGlow);
        }
        animateGlow();
    } else if (cursorGlow) {
        cursorGlow.style.display = 'none';
    }

    // Mobile nav toggle
    const navToggle = $('.nav-toggle');
    const navMenu = $('.nav-menu');
    navToggle?.addEventListener('click', () => {
        const isOpen = navMenu.classList.toggle('active');
        navToggle.classList.toggle('active');
        navToggle.setAttribute('aria-expanded', isOpen);
    });

    // Close nav on link click
    $$('.nav-link').forEach(link => {
        link.addEventListener('click', () => {
            navMenu.classList.remove('active');
            navToggle?.classList.remove('active');
        });
    });

    // Smooth scroll for anchor links
    $$('a[href^="#"]').forEach(a => {
        a.addEventListener('click', e => {
            const target = $(a.getAttribute('href'));
            if (target) {
                e.preventDefault();
                target.scrollIntoView({ behavior: 'smooth', block: 'start' });
            }
        });
    });

    // Counter animation
    const counters = $$('.metric-value[data-count]');
    const counterObserver = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                const el = entry.target;
                const target = parseInt(el.dataset.count);
                let current = 0;
                const increment = target / 40;
                const timer = setInterval(() => {
                    current += increment;
                    if (current >= target) {
                        el.textContent = target;
                        clearInterval(timer);
                    } else {
                        el.textContent = Math.floor(current);
                    }
                }, 30);
                counterObserver.unobserve(el);
            }
        });
    }, { threshold: 0.5 });
    counters.forEach(c => counterObserver.observe(c));

    // Fade-in animation for cards
    const fadeObserver = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.style.opacity = '1';
                entry.target.style.transform = 'translateY(0)';
                fadeObserver.unobserve(entry.target);
            }
        });
    }, { threshold: 0.1, rootMargin: '0px 0px -50px 0px' });

    $$('.bento-card, .tech-item').forEach((el, i) => {
        el.style.opacity = '0';
        el.style.transform = 'translateY(30px)';
        el.style.transition = `opacity 0.6s ease ${i * 0.1}s, transform 0.6s ease ${i * 0.1}s`;
        fadeObserver.observe(el);
    });

    // Year in footer
    const yearEl = $('#year');
    if (yearEl) yearEl.textContent = new Date().getFullYear();

    // Platform detection for download buttons
    const detectPlatform = () => {
        const ua = navigator.userAgent.toLowerCase();
        if (ua.includes('win')) return 'windows';
        if (ua.includes('mac')) return 'macos';
        if (ua.includes('linux')) return 'linux';
        return null;
    };

    const platform = detectPlatform();
    if (platform) {
        const btn = $(`.dl-btn[data-platform="${platform}"]`);
        if (btn) {
            btn.style.borderColor = 'var(--primary)';
            btn.style.background = 'rgba(99, 102, 241, 0.15)';
        }
    }

    // Download button click handlers
    $$('.dl-btn').forEach(btn => {
        btn.addEventListener('click', e => {
            e.preventDefault();
            const p = btn.dataset.platform;
            // Redirect to GitHub releases page
            window.location.href = 'https://github.com/vortex-interface/image/releases/latest';
        });
    });

    // Header hide on scroll
    const header = $('.header');
    let lastScroll = 0;
    window.addEventListener('scroll', () => {
        const scroll = window.scrollY;
        if (scroll > 100 && scroll > lastScroll) {
            header.style.transform = 'translateY(-100%)';
        } else {
            header.style.transform = 'translateY(0)';
        }
        lastScroll = scroll;
    }, { passive: true });

    // Console branding
    console.log('%ciMAGE', 'font-size:32px;font-weight:900;background:linear-gradient(135deg,#6366f1,#22d3ee);-webkit-background-clip:text;-webkit-text-fill-color:transparent;');
    console.log('%cNext-Gen Photo Storage', 'color:#71717a;');
})();
