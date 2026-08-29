import type { Meta, StoryObj } from '@storybook/svelte';
import AcerolaBookmarkRibbon from './acerola-bookmark-ribbon.svelte';

const meta = {
	title: 'Primitivos/AcerolaBookmarkRibbon',
	component: AcerolaBookmarkRibbon,
	tags: ['autodocs'],
	argTypes: {
		color: { control: 'number' },
		class: { control: 'text' }
	}
} satisfies Meta<typeof AcerolaBookmarkRibbon>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
	args: {
		color: 0xfff44336 // Red color
	}
};

export const BlueRibbon: Story = {
	args: {
		color: 0xff2196f3 // Blue color
	}
};

export const GreenRibbon: Story = {
	args: {
		color: 0xff4caf50 // Green color
	}
};
