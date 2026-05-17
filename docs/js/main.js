document.addEventListener('DOMContentLoaded', function () {
    initMobileMenu();
    initNavbarScroll();
    initSmoothScroll();
    initCopyButtons();
    initScrollAnimations();
    initPulsingDot();
    initScrollGuide();
    initHeaderToggles();
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
                    dotEl.textContent = '󱐱';
                    liveEl.textContent = 'LIVE ANALYSIS';
                    liveEl.style.color = '';
                } else {
                    dotEl.style.color = 'var(--danger)';
                    dotEl.textContent = '󱐱';
                    liveEl.textContent = 'PAUSED';
                    liveEl.style.color = 'var(--danger)';
                }
            }
        }
    });
}

function initScrollGuide() {
    var terminalWindow = document.getElementById('terminalWindow');
    var guideOverlay = document.getElementById('guideOverlay');
    var tuiApp = document.getElementById('tuiApp');
    var tourSection = document.getElementById('tourSection');
    var hero = document.getElementById('hero');
    var guideCards = document.querySelectorAll('.guide-card');

    if (!terminalWindow || !tourSection || !hero) return;

    function getTermWidth() {
        var vw = window.innerWidth;
        if (vw >= 1852) return Math.min(vw * 0.75, 1300);
        if (vw >= 1300) return Math.min(vw * 0.6, 1000);
        return Math.min(vw * 0.7, 850);
    }

    var stepNumEl = document.getElementById('stepNum');
    var stepLabelEl = document.getElementById('stepLabel');
    var stepDotsEl = document.getElementById('stepDots');

    
    if (stepDotsEl) {
        for (var i = 0; i < 10; i++) {
            var dot = document.createElement('span');
            dot.className = 'step-dot';
            stepDotsEl.appendChild(dot);
        }
    }

    var isFixed = false;
    var savedStyles = {};
    var firstFixDone = false;
    var currentStep = -1;
    var transitioning = false;

    var highlightClasses = [
        'highlight-header',
        'highlight-processes',
        'highlight-center',
        'highlight-risk',
        'highlight-timeline',
        'highlight-actions',
        'highlight-firewall',
        'highlight-search',
        'highlight-language',
        'highlight-investigation'
    ];

    var stepTargets = [
        'tuiHeader',
        'tuiLeft',
        'tuiCenter',
        'tuiCenter',
        'tuiCenter',
        'tuiRight',
        'firewallLayout',
        'searchOverlay',
        'langOverlay',
        'investigationView'
    ];

    var termStates = ['', '', '', 'risk', 'timeline', '', 'firewall', 'search', 'language', 'investigation'];

    var stepLabels = [
        'Dashboard Overview',
        'Process List',
        'Connections Table',
        'Risk Overview',
        'Timeline Chart',
        'Actions Panel',
        'Firewall Manager',
        'Live Search',
        'Multi-Language',
        'Connection Analysis'
    ];

    function saveStyles(el) {
        savedStyles = {
            position: el.style.position || '',
            top: el.style.top || '',
            left: el.style.left || '',
            width: el.style.width || '',
            transform: el.style.transform || '',
            zIndex: el.style.zIndex || '',
            margin: el.style.margin || '',
            transition: el.style.transition || '',
            opacity: el.style.opacity || ''
        };
    }

    function restoreStyles(el) {
        el.style.position = savedStyles.position;
        el.style.top = savedStyles.top;
        el.style.left = savedStyles.left;
        el.style.width = savedStyles.width;
        el.style.transform = savedStyles.transform;
        el.style.zIndex = savedStyles.zIndex;
        el.style.margin = savedStyles.margin;
        el.style.transition = savedStyles.transition;
        el.style.opacity = savedStyles.opacity;
        isFixed = false;
        firstFixDone = false;
        transitioning = false;
    }

    function fixAtCurrentPosition(el) {
        if (isFixed) return;
        var rect = el.getBoundingClientRect();
        var parentRect = el.parentElement ? el.parentElement.getBoundingClientRect() : rect;
        saveStyles(el);
        el.style.position = 'fixed';
        el.style.top = rect.top + 'px';
        el.style.left = rect.left + 'px';
        el.style.width = rect.width + 'px';
        el.style.margin = '0';
        el.style.zIndex = '150';
        el.style.transition = 'none';
        void el.offsetHeight;
        isFixed = true;
        firstFixDone = true;
    }

    function animateToCenter(el) {
        if (!isFixed) return;
        var tw = getTermWidth();
        el.style.transition = 'all 0.7s cubic-bezier(0.16, 1, 0.3, 1)';
        el.style.top = '50%';
        el.style.left = '50%';
        el.style.transform = 'translate(-50%, -50%)';
        el.style.width = tw + 'px';
        transitioning = true;
        var onEnd = function () {
            transitioning = false;
            el.removeEventListener('transitionend', onEnd);
        };
        el.addEventListener('transitionend', onEnd);
    }

    function animateBack(el) {
        if (!isFixed) return;
        var heroTerminal = document.querySelector('.hero-terminal');
        if (!heroTerminal) { restoreStyles(el); return; }
        var heroRect = heroTerminal.getBoundingClientRect();
        el.style.transition = 'all 0.5s cubic-bezier(0.16, 1, 0.3, 1)';
        el.style.top = heroRect.top + 'px';
        el.style.left = heroRect.left + 'px';
        el.style.width = heroRect.width + 'px';
        el.style.transform = 'none';
        transitioning = true;
        var onEnd = function () {
            el.style.transition = savedStyles.transition || '';
            el.style.position = savedStyles.position || '';
            el.style.top = savedStyles.top || '';
            el.style.left = savedStyles.left || '';
            el.style.width = savedStyles.width || '';
            el.style.transform = savedStyles.transform || '';
            el.style.margin = savedStyles.margin || '';
            el.style.zIndex = '';
            el.style.opacity = savedStyles.opacity || '1';
            el.style.display = '';
            isFixed = false;
            firstFixDone = false;
            transitioning = false;
            el.removeEventListener('transitionend', onEnd);
        };
        el.addEventListener('transitionend', onEnd);
        setTimeout(onEnd, 600);
    }

    function updateGuide(stepIndex) {
        if (stepIndex === currentStep) return;
        currentStep = stepIndex;

        guideCards.forEach(function (card, i) {
            card.classList.toggle('active', i === stepIndex);
        });

        
        var sidebarItems = document.querySelectorAll('.guide-step-item');
        sidebarItems.forEach(function (item, i) {
            item.classList.toggle('active-step', i === stepIndex);
            item.classList.toggle('completed-step', i < stepIndex);
        });

        
        delete tuiApp.dataset.termState;
        if (stepIndex >= 0 && stepIndex < termStates.length && termStates[stepIndex]) {
            tuiApp.dataset.termState = termStates[stepIndex];
        }

        
        var centerTabs = document.getElementById('centerTabs');
        if (centerTabs) {
            var tabs = centerTabs.querySelectorAll('.tab');
            tabs.forEach(function (t) { t.classList.remove('tab-active'); });
            var tabMap = { 2: 0, 3: 1, 4: 2 };
            var tabIdx = tabMap[stepIndex];
            if (tabIdx !== undefined && tabs[tabIdx]) {
                tabs[tabIdx].classList.add('tab-active');
            } else {
                
                tabs[0].classList.add('tab-active');
            }
        }

        tuiApp.classList.remove('guide-active', 'dim-others');
        highlightClasses.forEach(function (c) { tuiApp.classList.remove(c); });

        if (stepIndex >= 0 && stepIndex < highlightClasses.length) {
            tuiApp.classList.add('guide-active', 'dim-others', highlightClasses[stepIndex]);

            var targetId = stepTargets[stepIndex];
            var targetEl = document.getElementById(targetId);
            if (targetEl) {
                targetEl.classList.add('highlighted');
                var parent = targetEl.parentElement;
                
                var scope = parent;
                if (termStates[stepIndex] === 'firewall') {
                    scope = document.getElementById('firewallLayout');
                } else if (termStates[stepIndex] === 'search') {
                    scope = document.getElementById('tuiApp');
                } else if (termStates[stepIndex] === 'language') {
                    scope = document.getElementById('tuiApp');
                } else if (termStates[stepIndex] === 'investigation') {
                    scope = document.getElementById('investigationView');
                } else if (termStates[stepIndex] === 'map') {
                    scope = document.getElementById('mapView');
                }
                if (scope) {
                    var siblings = scope.querySelectorAll('.tui-panel, .tui-header, .tui-hintbar, .tui-statusbar, .fw-col, .fw-actions, .search-overlay, .lang-overlay, .investigation-view, .map-view, .inv-hintbar, .inv-grid, .inv-left, .inv-right, .map-body');
                    siblings.forEach(function (s) {
                        if (s !== targetEl) s.classList.remove('highlighted');
                    });
                }
            }
        }

        
        if (stepNumEl && stepLabelEl) {
            if (stepIndex >= 0) {
                var num = (stepIndex + 1).toString().padStart(2, '0');
                stepNumEl.textContent = num;
                stepLabelEl.textContent = stepLabels[stepIndex] || '';
                if (stepDotsEl) {
                    var dots = stepDotsEl.querySelectorAll('.step-dot');
                    dots.forEach(function (d, i) {
                        d.className = 'step-dot';
                        if (i === stepIndex) d.classList.add('active');
                        else if (i < stepIndex) d.classList.add('done');
                    });
                }
            }
        }
    }

    function handleScroll() {
        var scrollY = window.scrollY;
        var winH = window.innerHeight;
        var heroH = hero.offsetHeight;
        var tourH = tourSection.offsetHeight;
        var tourTop = heroH;
        var tourEnd = heroH + tourH;

        var transitionStart = Math.max(0, heroH - winH * 0.4);
        var transitionEnd = heroH + 100;

        
        if (scrollY < transitionStart) {
            if (isFixed) {
                guideOverlay.style.display = 'none';
                guideOverlay.classList.remove('guide-visible');
                animateBack(terminalWindow);
                updateGuide(-1);
            }
            guideOverlay.style.display = 'none';
            guideOverlay.classList.remove('guide-visible');
            tuiApp.dataset.termState = 'loading';
            return;
        }

        
        if (scrollY < transitionEnd) {
            guideOverlay.style.display = 'none';
            guideOverlay.classList.remove('guide-visible');
            updateGuide(-1);
            tuiApp.dataset.termState = 'loading';

            if (!isFixed && !transitioning) {
                fixAtCurrentPosition(terminalWindow);
                var self = terminalWindow;
                requestAnimationFrame(function () {
                    animateToCenter(self);
                });
            }
            return;
        }

        
        if (scrollY < tourEnd - winH * 0.6) {
            guideOverlay.style.display = 'block';
            guideOverlay.classList.add('guide-visible');

            if (!isFixed && !transitioning) {
                fixAtCurrentPosition(terminalWindow);
                animateToCenter(terminalWindow);
            }

            if (isFixed) {
                var tw = getTermWidth();
                terminalWindow.style.top = '50%';
                terminalWindow.style.left = '50%';
                terminalWindow.style.transform = 'translate(-50%, -50%)';
                terminalWindow.style.width = tw + 'px';
                transitioning = false;
            }

            var guideStart = transitionEnd;
            var guideRange = (tourEnd - winH * 0.6) - guideStart;
            var progress = guideRange > 0 ? (scrollY - guideStart) / guideRange : 0;
            progress = Math.max(0, Math.min(1, progress));

            var stepIndex = Math.min(Math.floor(progress * 10), 9);
            updateGuide(stepIndex);

            terminalWindow.style.display = '';
            terminalWindow.style.opacity = '1';
            return;
        }

        
        guideOverlay.style.display = 'none';
        guideOverlay.classList.remove('guide-visible');
        updateGuide(-1);
        tuiApp.dataset.termState = 'loading';
        if (isFixed) {
            restoreStyles(terminalWindow);
        }
    }

    var ticking = false;
    window.addEventListener('scroll', function () {
        if (!ticking) {
            requestAnimationFrame(function () {
                handleScroll();
                ticking = false;
            });
            ticking = true;
        }
    }, { passive: true });

    window.addEventListener('resize', function () {
        if (isFixed) {
            terminalWindow.style.top = '50%';
            terminalWindow.style.left = '50%';
            terminalWindow.style.transform = 'translate(-50%, -50%)';
            terminalWindow.style.width = getTermWidth() + 'px';
        }
    });

    tuiApp.dataset.termState = 'loading';
    handleScroll();
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