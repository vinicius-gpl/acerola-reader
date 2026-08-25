import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export {
	type WithElementRef,
	type WithoutChild,
	type WithoutChildren,
	type WithoutChildrenOrChild
} from 'bits-ui';

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}
