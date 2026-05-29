document.addEventListener('DOMContentLoaded', function () {
    initMobileMenu();
    initNavbarScroll();
    initSmoothScroll();
    initCopyButtons();
    initScrollAnimations();
    initPulsingDot();
    initScrollGuide();
    initHeaderToggles();
    initNavViewSwitching();
    initFeaturesToggle();
});

function initMobileMenu() {
    var btn = document.getElementById('mobileMenuBtn');
    var nav = document.querySelector('.nav-links');
    if (btn && nav) {
        btn.addEventListener('click', function () {
            nav.classList.toggle('active');
            btn.classList.toggle('active');
        });
        nav.querySelectorAll('a').forEach(function (link) {
            link.addEventListener('click', function () {
                nav.classList.remove('active');
                btn.classList.remove('active');
            });
        });
    }
}

function initNavbarScroll() {
    var navbar = document.getElementById('navbar');
    if (navbar) {
        window.addEventListener('scroll', function () {
            navbar.classList.toggle('scrolled', window.scrollY > 50);
        });
    }
}

function initSmoothScroll() {
    document.querySelectorAll('a[href^="#"]').forEach(function (anchor) {
        anchor.addEventListener('click', function (e) {
            e.preventDefault();
            var targetId = this.getAttribute('href');
            if (targetId === '#') return;
            var targetElement = document.querySelector(targetId);
            if (targetElement) {
                var navHeight = document.getElementById('navbar').offsetHeight;
                var targetPosition = targetElement.getBoundingClientRect().top + window.pageYOffset - navHeight;
                window.scrollTo({ top: targetPosition, behavior: 'smooth' });
            }
        });
    });
}

function initCopyButtons() {
    document.querySelectorAll('.copy-btn').forEach(function (btn) {
        btn.addEventListener('click', function () {
            var text = this.getAttribute('data-copy');
            if (!text) return;
            navigator.clipboard.writeText(text).then(function () {
                var orig = this.textContent;
                this.textContent = 'Copied!';
                this.style.color = '#32cd32';
                var self = this;
                setTimeout(function () { self.textContent = orig; self.style.color = ''; }, 2000);
            }.bind(this)).catch(function () {
                var ta = document.createElement('textarea');
                ta.value = text;
                document.body.appendChild(ta);
                ta.select();
                document.execCommand('copy');
                document.body.removeChild(ta);
                this.textContent = 'Copied!';
                var self = this;
                setTimeout(function () { self.textContent = 'Copy'; }, 2000);
            }.bind(this));
        });
    });
}

function initScrollAnimations() {
    var observer = new IntersectionObserver(function (entries) {
        entries.forEach(function (entry) {
            if (entry.isIntersecting) {
                var delay = entry.target.getAttribute('data-aos-delay');
                if (delay) {
                    setTimeout(function () { entry.target.classList.add('aos-animate'); }, parseInt(delay));
                } else {
                    entry.target.classList.add('aos-animate');
                }
            }
        });
    }, { root: null, rootMargin: '0px', threshold: 0.1 });

    document.querySelectorAll('[data-aos]').forEach(function (el) {
        observer.observe(el);
    });
}

function initPulsingDot() {
    
}

function initHeaderToggles() {
    var header = document.getElementById('tuiHeader');
    if (!header) return;

    
    document.addEventListener('keydown', function (e) {
        if (e.key === 'h' || e.key === 'H') {
            if (document.querySelector('.guide-overlay.guide-visible')) return;
            var isHunter = header.dataset.hunter === 'true';
            header.dataset.hunter = isHunter ? 'false' : 'true';
            var modeEl = document.getElementById('headerMode');
            if (modeEl) {
                modeEl.textContent = isHunter ? '󰒓 NORMAL' : '󰒓 HUNTER';
                modeEl.style.color = isHunter ? '' : 'var(--success)';
            }
        }
    });

    
    document.addEventListener('keydown', function (e) {
        if (e.key === 'p' || e.key === 'P') {
            if (document.querySelector('.guide-overlay.guide-visible')) return;
            var isPaused = header.dataset.paused === 'true';
            header.dataset.paused = isPaused ? 'false' : 'true';
            var dotEl = document.getElementById('headerDot');
            var liveEl = document.getElementById('headerLive');
            if (dotEl && liveEl) {
                if (isPaused) {
                    dotEl.style.color = '';
                    dotEl.textContent = '\uf004';
                    liveEl.textContent = 'LIVE ANALYSIS';
                    liveEl.style.color = '';
                } else {
                    dotEl.style.color = 'var(--danger)';
                    dotEl.textContent = '\uf004';
                    liveEl.textContent = 'PAUSED';
                    liveEl.style.color = 'var(--danger)';
                }
            }
        }
    });
}

function initScrollGuide() {
    const terminal = document.getElementById('terminalWindow');
    const steps = document.querySelectorAll('.tour-step');
    const tuiApp = document.getElementById('tuiApp');
    const tuiBody = document.getElementById('tuiBody');
    const hero = document.getElementById('hero');

    if (!terminal || !steps.length || !tuiApp) return;

    // Mapping steps to TUI states
    const termStates = [
        'main', 'main', 'main', 'main', 'risk', 'timeline', 'main', 'trends', 'storage', 'libraries', 'containers', 'firewall', 'search', 'language', 'investigation',
        'trends', 'storage', 'containers' // New steps
    ];

    const navMap = [
        'main', 'main', 'main', 'main', 'main', 'main', 'main', 'trends', 'storage', 'libraries', 'containers', 'main', 'main', 'main', 'main',
        'trends', 'storage', 'containers' // New steps
    ];

    const highlightClasses = [
        'highlight-nav-sidebar', 'highlight-header', 'highlight-processes', 'highlight-center', 'highlight-risk', 'highlight-timeline', 'highlight-actions', 'highlight-trends', 'highlight-storage', 'highlight-libraries', 'highlight-containers', 'highlight-firewall', 'highlight-search', 'highlight-language', 'highlight-investigation',
        'highlight-trends', 'highlight-storage', 'highlight-containers'
    ];

    let isFixed = false;
    let originalParent = terminal.parentElement;

    const observerOptions = {
        root: null,
        rootMargin: '-30% 0px -30% 0px', // Adjusted for longer scroll space
        threshold: 0
    };

    const stepObserver = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                const stepIndex = parseInt(entry.target.dataset.step);
                activateStep(stepIndex);
            }
        });
    }, observerOptions);

    steps.forEach(step => stepObserver.observe(step));

    function activateStep(index) {
        // Update active class on steps
        steps.forEach((s, i) => s.classList.toggle('active', i === index));

        // Update Nav View
        const targetNav = navMap[index] || 'main';
        if (tuiBody && tuiBody.dataset.activeNav !== targetNav) {
            tuiBody.dataset.activeNav = targetNav;
            document.querySelectorAll('.nav-item').forEach(ni => {
                ni.classList.toggle('nav-selected', ni.dataset.nav === targetNav);
            });
        }

        // Highlight panels - Remove all highlight classes first
        tuiApp.classList.remove(...highlightClasses);
        tuiApp.classList.add('guide-active', 'dim-others');
        if (highlightClasses[index]) {
            tuiApp.classList.add(highlightClasses[index]);
        }

        // Tab switching logic for center-tabs
        const centerTabs = document.querySelectorAll('.center-tabs .tab');
        if (centerTabs.length === 3) {
            centerTabs.forEach(t => t.classList.remove('tab-active'));
            centerTabs[0].textContent = '[1] Connections';
            centerTabs[1].textContent = '[2] Risk Overview';
            centerTabs[2].textContent = '[3] Timeline';

            if (termStates[index] === 'risk') {
                centerTabs[1].classList.add('tab-active');
                centerTabs[1].textContent = '▎[2] Risk Overview';
            } else if (termStates[index] === 'timeline') {
                centerTabs[2].classList.add('tab-active');
                centerTabs[2].textContent = '▎[3] Timeline';
            } else {
                centerTabs[0].classList.add('tab-active');
                centerTabs[0].textContent = '▎[1] Connections';
            }
        }

        // Update TUI State
        delete tuiApp.dataset.termState;
        if (termStates[index]) tuiApp.dataset.termState = termStates[index];
        
        // Update step indicator in TUI
        const stepNumEl = document.getElementById('stepNum');
        const stepLabelEl = document.getElementById('stepLabel');
        if (stepNumEl) stepNumEl.textContent = (index + 1).toString().padStart(2, '0');
        if (stepLabelEl) {
            const labels = ["Navigation", "Dashboard", "Processes", "Connections", "Risk", "Timeline", "Actions", "Trends", "Storage", "Libraries", "Containers", "Firewall", "Search", "Language", "Investigation"];
            stepLabelEl.textContent = labels[index] || "";
        }
    }

    // Handle the movement of terminal from Hero to Sticky Tour
    window.addEventListener('scroll', () => {
        const scrollY = window.scrollY;
        const heroH = hero.offsetHeight;
        const tourSection = document.getElementById('tourSection');
        const stickyWrapper = document.getElementById('stickyWrapper');
        const featuresSec = document.getElementById('features');
        
        if (!tourSection) return;

        // Hide terminal when Powerful Features section enters viewport
        if (featuresSec) {
            const featuresTop = featuresSec.offsetTop;
            if (scrollY + window.innerHeight > featuresTop + 100) {
                if (isFixed) {
                    originalParent.appendChild(terminal);
                    isFixed = false;
                    tuiApp.classList.remove('guide-active', 'dim-others', ...highlightClasses);
                    delete tuiApp.dataset.termState;
                }
                terminal.style.display = 'none';
                return;
            } else {
                terminal.style.display = '';
            }
        }
        
        const tourTop = tourSection.offsetTop;
        const tourBottom = tourTop + tourSection.offsetHeight;

        if (scrollY > tourTop - 100 && scrollY < tourBottom - 800) {
            if (!isFixed) {
                stickyWrapper.appendChild(terminal);
                isFixed = true;
                terminal.style.animation = 'none';
                terminal.style.opacity = '1';
                terminal.style.transform = 'none';
                tuiApp.classList.add('guide-active');
            }
        } else {
            if (isFixed) {
                originalParent.appendChild(terminal);
                isFixed = false;
                tuiApp.classList.remove('guide-active', 'dim-others', ...highlightClasses);
                delete tuiApp.dataset.termState;
            }
        }
    });
}


function initFeaturesToggle() {
    var btn = document.getElementById('featuresToggle');
    var extra = document.getElementById('featuresExtra');
    var icon = document.getElementById('toggleIcon');
    var text = document.getElementById('toggleText');
    if (!btn || !extra) return;
    btn.addEventListener('click', function () {
        var expanded = extra.classList.toggle('expanded');
        btn.classList.toggle('expanded');
        icon.textContent = expanded ? '×' : '+';
        text.textContent = expanded ? 'Hide features' : 'Show all features';
    });
}

function initNavViewSwitching() {
    var navItems = document.querySelectorAll('.nav-item');
    var tuiBody = document.getElementById('tuiBody');
    if (!navItems.length || !tuiBody) return;

    navItems.forEach(function (item) {
        item.addEventListener('click', function () {
            var nav = this.dataset.nav;
            if (!nav) return;
            tuiBody.dataset.activeNav = nav;
            navItems.forEach(function (ni) {
                ni.classList.toggle('nav-selected', ni.dataset.nav === nav);
            });
            delete document.getElementById('tuiApp').dataset.termState;
        });
    });

    document.addEventListener('keydown', function (e) {
        if (e.key === 'm' || e.key === 'M') {
            var navSidebar = document.getElementById('tuiNavSidebar');
            if (navSidebar) {
                navSidebar.classList.toggle('nav-collapsed');
            }
        }
    });
}


(function () {
    var code = ['ArrowUp', 'ArrowUp', 'ArrowDown', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'b', 'a'];
    var idx = 0;
    document.addEventListener('keydown', function (e) {
        if (e.key === code[idx]) {
            idx++;
            if (idx === code.length) {
                document.body.style.background = 'linear-gradient(45deg, #6465ed, #ffa500, #32cd32)';
                document.body.style.backgroundSize = '400% 400%';
                document.body.style.animation = 'gradientShift 10s ease infinite';
                var style = document.createElement('style');
                style.textContent = '@keyframes gradientShift { 0%{background-position:0% 50%} 50%{background-position:100% 50%} 100%{background-position:0% 50%} }';
                document.head.appendChild(style);
                idx = 0;
            }
        } else {
            idx = 0;
        }
    });
})();