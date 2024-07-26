// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const awaiter = async <T>(promise: Promise<T>): Promise<[T | null, any | null]> => {
	try {
		const data: T = await promise;
		return [data, null];
	} catch (err) {
		return [null, err];
	}
};
