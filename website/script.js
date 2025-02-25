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
