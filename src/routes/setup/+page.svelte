<script lang="ts">
	let loading = true;
	let errored = false;
	let step = 0;
	let status = 'Contacting Fansly and checking account...';

	let fanslyToken = '';
	let validationErrors = {
		fanslyToken: ''
	};

	import { invoke } from '@tauri-apps/api/core';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { message } from '@tauri-apps/plugin-dialog';
	import { awaiter } from '$lib/utils';
	import { onMount } from 'svelte';
	import type { AccountInfo, Config } from '$lib/types';

	onMount(async () => {
		const [config, configError] = await awaiter(invoke('get_config'));

		if (configError) {
			await message(
				`Something went wrong while getting the configuration. We've copied the error to your clipboard. Please report this issue on GitHub.\n\nError: ${configError}\n\nThe application will now close.`,
				{ title: 'BambuConnect | Configuration Error', kind: 'error' }
			);

			await writeText(configError);
			invoke('quit', { code: 1 });
			return;
		}

		loading = false;
	});

	async function loginToFanslyAndFetchData() {
		const [config, configError] = (await awaiter(invoke('get_config'))) as [Config, any | null];

		if (configError) {
			await message(
				`Something went wrong while getting the configuration. We've copied the error to your clipboard. Please report this issue on GitHub.\n\nError: ${configError}\n\nThe application will now close.`,
				{ title: 'BambuConnect | Configuration Error', kind: 'error' }
			);

			await writeText(configError);
			invoke('quit', { code: 1 });
			return;
		}
		await invoke('fansly_set_token', { token: fanslyToken });
		const [me, error] = (await awaiter(invoke('fansly_get_me'))) as [
			{
				success: boolean;
				response: AccountInfo;
			},
			any | null
		];

		if (me === null) {
			console.error(`Failed to authenticate with Fansly. Error: ${error}`);
			step = 1;
			validationErrors.fanslyToken =
				'We could not authenticate with Fansly. Please check your token and try again.';
			return;
		}

		if (!me.success) {
			console.error(`Failed to authenticate with Fansly. Error: ${me.response}`);
			step = 1;
			validationErrors.fanslyToken =
				'We could not authenticate with Fansly. Please check your token and try again.';
			return;
		}

		status = 'Finishing up...';

		console.log('fanslyToken', fanslyToken);
		config.fansly_token = fanslyToken;
		config.is_first_run = false;
		await invoke('save_config', { config });
		step = 3;
	}

	function validateLoginForm() {
		let hasErrors = false;

		if (fanslyToken === '') {
			validationErrors.fanslyToken = 'Please enter your Fansly token.';
			hasErrors = true;
		} else {
			validationErrors.fanslyToken = '';
		}

		if (hasErrors) {
			return;
		}

		step = 2;
		loginToFanslyAndFetchData();
	}
</script>

<div class="container bg-zinc-800 w-screen h-screen">
	<!-- Centered loading spinner -->
	<div class="flex flex-col justify-center items-center h-screen">
		{#if loading}
			<div role="status">
				<svg
					aria-hidden="true"
					class="w-14 h-14 text-gray-200 animate-spin dark:text-gray-600 fill-blue-600"
					viewBox="0 0 100 101"
					fill="none"
					xmlns="http://www.w3.org/2000/svg"
				>
					<path
						d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z"
						fill="currentColor"
					/>
					<path
						d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z"
						fill="currentFill"
					/>
				</svg>
				<span class="sr-only">Loading...</span>
			</div>

			<h1 class="text-2xl font-bold mt-2 text-white">Setup is loading...</h1>
		{/if}

		{#if errored}
			<svg
				xmlns="http://www.w3.org/2000/svg"
				fill="none"
				viewBox="0 0 24 24"
				stroke-width="1.5"
				stroke="currentColor"
				class="w-12 h-12 text-red-500 dark:text-red-400"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z"
				/>
			</svg>

			<h1 class="text-2xl font-bold mt-2 text-white">Oops!</h1>
			<p class="text-red-500 mt-2 max-w-[30em]">
				An error was encountered while initializing the setup, please try closing and reopening
				FanslySync.
			</p>
		{/if}

		{#if !loading && !errored}
			{#if step === 0}
				<h1 class="text-2xl font-bold mt-2 text-white">Welcome to FanslySync!</h1>
				<p class="text-gray-200 break-words max-w-[30em]">
					Because this is your first time running FanslySync, we need to set up the connection to
					your Fansly account. Click the button below to get started.
				</p>

				{#if errored}
					<p class="text-red-500 mt-2">
						An error occurred while initializing the setup. Please try again.
					</p>
				{/if}

				{#if !loading}
					<button
						on:click={() => {
							step = 1;
						}}
						class="mt-4 bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 transition-all duration-200 ease-in-out"
					>
						Begin Setup
					</button>
				{/if}
			{:else if step === 1}
				<h1 class="text-2xl font-bold mt-2 text-white">Authenticate with Fansly</h1>
				<p class="text-gray-200 break-words max-w-[30em]">
					To establish a secure connection with Fansly, we require your Fansly Authentication Token.
					We do not transmit this token to our servers and it is only used to authenticate with
					Fansly and fetch data locally. <br /> <br />
					For more information on how to get your Fansly Authentication Token, please visit our documentation,
					or join our Discord server if you need help or have any questions.
				</p>

				<label for="username" class="text-gray-200 mt-4"> Fansly Authentication Token </label>
				<input
					id="fanslyToken"
					class="w-full bg-zinc-700 text-gray-200 px-4 py-2 rounded-md mt-2 max-w-96"
					type="text"
					autocomplete="current-password"
					bind:value={fanslyToken}
				/>

				{#if validationErrors.fanslyToken !== ''}
					<p class="text-red-500 mt-2">{validationErrors.fanslyToken}</p>
				{/if}

				{#if !loading}
					<button
						on:click={validateLoginForm}
						class="mt-4 bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 transition-all duration-200 ease-in-out"
					>
						Authenticate and setup connection
					</button>
				{/if}
			{:else if step == 2}
				<div role="status">
					<svg
						aria-hidden="true"
						class="w-14 h-14 text-gray-200 animate-spin dark:text-gray-600 fill-blue-600"
						viewBox="0 0 100 101"
						fill="none"
						xmlns="http://www.w3.org/2000/svg"
					>
						<path
							d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z"
							fill="currentColor"
						/>
						<path
							d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z"
							fill="currentFill"
						/>
					</svg>
					<span class="sr-only">Loading...</span>
				</div>

				<h1 class="text-2xl font-bold mt-2 text-white">We are working our magic!</h1>
				<p class="text-gray-200 break-words max-w-[30em]">
					We are now authenticating with Fansly and fetching your data. This may take a few moments.
				</p>
				<p class="text-gray-200 break-words max-w-[30em]">
					{status}
				</p>
			{:else if step === 3}
				<svg
					xmlns="http://www.w3.org/2000/svg"
					fill="none"
					viewBox="0 0 24 24"
					stroke-width="1.5"
					stroke="currentColor"
					class="w-12 h-12 text-green-500 dark:text-green-400"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
					/>
				</svg>
				<h1 class="text-2xl font-bold mt-2 text-white">Setup Complete!</h1>
				<p class="text-gray-200 break-words max-w-[30em]">
					You're all set! FanslySync is now connected to your Fansly account and is ready to use.
				</p>

				<button
					on:click={() => {
						window.location.href = '/home';
					}}
					class="mt-4 bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 transition-all duration-200 ease-in-out"
				>
					Finish
				</button>
			{/if}
		{/if}
	</div>
</div>
