// Simple animation: Bounce effect on team member photos when hovered
document.querySelectorAll('.member').forEach(member => {
    member.addEventListener('mouseover', () => {
        const photo = member.querySelector('.photo-placeholder');
        photo.style.transform = 'translateY(-10px)';
        photo.style.transition = 'transform 0.3s ease';
    });

    member.addEventListener('mouseout', () => {
        const photo = member.querySelector('.photo-placeholder');
        photo.style.transform = 'translateY(0)';
    });
});

// Fonction pour détecter si un élément est visible dans le viewport
function isElementInViewport(el) {
    const rect = el.getBoundingClientRect();
    return (
        rect.top >= 0 &&
        rect.left >= 0 &&
        rect.bottom <= (window.innerHeight || document.documentElement.clientHeight) &&
        rect.right <= (window.innerWidth || document.documentElement.clientWidth)
    );
}

// Fonction pour gérer les animations au scroll
function handleScrollAnimations() {
    const sections = document.querySelectorAll('.section-animate');
    
    sections.forEach(section => {
        if (isElementInViewport(section)) {
            section.classList.add('visible');
        }
    });
}

// Ajouter les écouteurs d'événements
document.addEventListener('DOMContentLoaded', () => {
    // Initialiser les animations au chargement
    handleScrollAnimations();
    
    // Ajouter l'écouteur de scroll
    window.addEventListener('scroll', handleScrollAnimations);
});

// Animation douce pour les liens d'ancrage
document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function (e) {
        e.preventDefault();
        const target = document.querySelector(this.getAttribute('href'));
        if (target) {
            target.scrollIntoView({
                behavior: 'smooth'
            });
        }
    });
});
