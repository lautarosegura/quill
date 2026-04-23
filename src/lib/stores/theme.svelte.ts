type Theme = 'dark' | 'light';

let current = $state<Theme>('dark');

function apply(theme: Theme) {
	if (typeof document !== 'undefined') {
		const html = document.documentElement;
		if (theme === 'dark') html.classList.add('dark');
		else html.classList.remove('dark');
	}
}

export const theme = {
	get value() {
		return current;
	},
	toggle() {
		current = current === 'dark' ? 'light' : 'dark';
		apply(current);
	},
	init() {
		apply(current);
	}
};
