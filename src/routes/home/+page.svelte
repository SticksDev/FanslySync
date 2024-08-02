<script lang="ts">
	import { info, error } from '@tauri-apps/plugin-log';
	import { awaiter } from '$lib/utils';
	import { onMount } from 'svelte';
	import type { Config, SyncData } from '$lib/types';
	import { slide } from 'svelte/transition';
	import { sendNotification } from '@tauri-apps/plugin-notification';
	import { platform } from '@tauri-apps/plugin-os';
	import { check, Update } from '@tauri-apps/plugin-updater';
	import { getVersion, getTauriVersion } from '@tauri-apps/api/app';
	import { invoke } from '@tauri-apps/api/core';
	import { ask, message } from '@tauri-apps/plugin-dialog';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';

	let loadingSync = true;
	let upToDate: boolean | null = null;
	let updateData: Update | null = null;
	let versionData = {
		appVersion: 'Loading...',
		tauriVersion: 'Loading...'
	};
	let syncState = {
		show: false,
		syncing: false,
		error: false,
		success: false,
		url: '',
		message: ''
	};

	let config: Config | null = null;

	onMount(async () => {
		info(`[FanslySync::page_init:home] onMount() called. Starting page initialization...`);
		const [configData, configError] = await awaiter(invoke('get_config') as Promise<Config>);

		if (configError || !configData) {
			error(
				`[FanslySync::page_init:home] Failed to get configuration. Error: ${configError ?? 'Config was null'}`
			);
			info(
				`[FanslySync::page_init:home] Page Initialization failed due to configuration error. Exiting...`
			);
			await message(
				`Something went wrong while getting the configuration. We've copied the error to your clipboard. Please report this issue on GitHub.\n\nError: ${configError ?? 'Config was null'}\n\nThe application will now close.`,
				{ title: 'FanslySync | Configuration Error', kind: 'error' }
			);

			await writeText(configError);
			invoke('quit', { code: 1 });
			return;
		}

		info(
			`[FanslySync::page_init:home] Configuration initialized successfully. Checking for updates...`
		);
		config = configData;
		loadingSync = false;

		const updateStatus = await check();
		upToDate = !updateStatus?.available ?? false;
		updateData = updateStatus;
		info(
			`[FanslySync::page_init:home] Update check completed. Up to date: ${upToDate}, Update data: ${updateData}`
		);
		info(`[FanslySync::page_init:home] Getting app and Tauri versions...`);

		versionData.appVersion = await getVersion();
		versionData.tauriVersion = await getTauriVersion();

		info(
			`[FanslySync::page_init:home] App and Tauri versions fetched. We are running App version: ${versionData.appVersion}, atop Tauri version: ${versionData.tauriVersion}`
		);

		info(`[FanslySync::page_init:home] Page initialization completed successfully.`);
	});

	async function syncNow() {
		info(`[FanslySync::syncNow] Starting manual sync...`);

		syncState.error = false;
		syncState.success = false;
		syncState.syncing = true;
		syncState.show = true;

		const [syncData, syncError] = await awaiter(invoke('fansly_sync') as Promise<SyncData>);
		console.log(syncData, syncError);

		if (syncError || syncData === null) {
			error(
				`[FanslySync::syncNow] Failed to sync data. Error: ${syncError ?? 'Sync data was null'}`
			);
			syncState.syncing = false;
			syncState.error = true;
			syncState.message = syncError ?? 'Sync data was null';
			return;
		}

		syncState.url = syncData.sync_data_url;

		// Return the last sync as unix timestamp
		config!.last_sync = Date.now();
		config!.last_sync_data = syncData!;

		const [saveConfigData, saveConfigError] = await awaiter(
			invoke('save_config', { config }) as Promise<boolean>
		);
		if (saveConfigError) {
			error(`[FanslySync::syncNow] Failed to save config data. Error: ${saveConfigError}`);
			syncState.syncing = false;
			syncState.error = true;
			syncState.message = saveConfigError ?? 'Save config data was null';
			return;
		}

		syncState.syncing = false;
		syncState.success = true;

		const platformName = platform();
		let soundName;

		if (platformName === 'windows') {
			soundName = 'ms-winsoundevent:Notification.Default';
		} else if (platformName === 'macos') {
			soundName = 'Ping';
		} else {
			soundName = 'completion-sucess';
		}

		await sendNotification({
			title: 'FanslySync: Sync Successful!',
			body: 'Data synced successfully. Please look at the app for more details.',
			sound: soundName
		});

		info(`[FanslySync::syncNow] Manual sync completed successfully.`);
	}

	async function doUpdate() {
		if (!updateData || !updateData.available) {
			message('You are up to date! Current version: ' + versionData.appVersion, {
				title: 'FanslySync | Update',
				kind: 'info'
			});
			return;
		}

		const confirm = await ask(
			`An update is available for FanslySync. Would you like to update now?\n\nCurrent version: ${versionData.appVersion}\nNew (remote) version: ${updateData?.version ?? 'Unknown'}\n\nThe application will close after the update completes.`,
			{
				title: 'Update Available',
				okLabel: 'Yes',
				cancelLabel: 'No',
				kind: 'info'
			}
		);

		if (confirm) {
			await updateData.downloadAndInstall();
			await invoke('quit', { code: 0 });
		} else {
			console.log('User declined update');
		}
	}
</script>

<div class="container bg-zinc-800 w-screen h-screen">
	<!-- Top header, Bambu Connect at the left side, settings icon on the right -->
	<div class="flex justify-between items-center h-16 px-4 bg-zinc-900">
		<div class="flex items-center">
			<img src="/fanslySync.png" alt="FanslySynct" class="w-8 h-8" />
			<h1 class="text-2xl font-bold text-gray-200 ml-2">FanslySync</h1>
			<span class="text-gray-400 ml-2"
				>v{versionData.appVersion} (runtime: {versionData.tauriVersion})</span
			>
			{#if upToDate === false}
				<button
					type="button"
					class="text-red-500 ml-2 hover:text-red-600 duration-200"
					on:click={doUpdate}
				>
					Update available!
				</button>
			{:else if upToDate === true}
				<span class="text-green-500 ml-2">Up to date!</span>
			{/if}
		</div>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			fill="none"
			viewBox="0 0 24 24"
			stroke-width="1.5"
			stroke="currentColor"
			class="w-6 h-6 text-white hidden"
		>
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7.723 7.723 0 0 1 0 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.247a1.125 1.125 0 0 1-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.47 6.47 0 0 1-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.94-1.11.94h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 0 1-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 0 1-1.369-.49l-1.297-2.247a1.125 1.125 0 0 1 .26-1.431l1.004-.827c.292-.24.437-.613.43-.991a6.932 6.932 0 0 1 0-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 0 1-.26-1.43l1.297-2.247a1.125 1.125 0 0 1 1.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28Z"
			/>
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"
			/>
		</svg>
	</div>

	<!-- Sync container -->
	<div class="flex flex-col mt-4 px-3">
		<h2 class="text-2xl font-bold text-gray-200">Sync</h2>
		<p class="text-gray-400 mt-1">Manage automatic sync options and manual sync here.</p>
		<div class="flex flex-wrap mt-2 rounded-lg">
			{#if loadingSync}
				<div class="flex flex-col items-center justify-center">
					<div role="status" class="mb-2">
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

					<p class="text-gray-400 animate-pulse">Loading sync options, one moment...</p>
				</div>
			{:else}
				<div class="flex flex-col gap-y-2 justify-start">
					<!-- Automatic sync card -->
					<div class="relative">
						<div class="bg-zinc-700 p-4 rounded-lg">
							<h1 class="text-xl font-bold text-gray-200">Automatic Sync</h1>
							<p class="text-gray-400 mt-1">
								Sync content automatically every {config?.sync_interval} hours. Please ensure you have
								a stable internet connection.
							</p>
							<div class="flex mt-2">
								<button
									class="bg-blue-600 text-white px-4 py-2 rounded-lg w-full"
									on:click={() => console.log('Automatic sync clicked')}
								>
									Enable
								</button>
							</div>
						</div>
						<div
							class="absolute top-0 left-0 right-0 bottom-0 bg-white/30 backdrop-blur-sm flex flex-col justify-center items-center rounded-lg"
						>
							<svg
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="1.5"
								stroke="currentColor"
								class="size-10 text-blue-400"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="m11.25 11.25.041-.02a.75.75 0 0 1 1.063.852l-.708 2.836a.75.75 0 0 0 1.063.853l.041-.021M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9-3.75h.008v.008H12V8.25Z"
								/>
							</svg>
							<h1 class="text-xl font-bold text-gray-200 ml-2">Automatic Sync is coming soon!</h1>
							<p class="text-white mt-1">Stay tuned for updates.</p>
						</div>
					</div>

					<!-- Manual sync card -->
					<div class=" bg-zinc-700 p-4 rounded-lg">
						<h1 class="text-xl font-bold text-gray-200">Manual Sync</h1>
						<p class="text-gray-400 mt-1">
							Trigger a manual sync now, instead of waiting for an automatic sync.
						</p>
						<div class="flex mt-2">
							<button
								class="bg-blue-600 text-white px-4 py-2 rounded-lg w-full disabled:opacity-50 disabled:cursor-not-allowed hover:bg-blue-700 transition-all duration-200 ease-in-out"
								on:click={syncNow}
								disabled={syncState.syncing}
							>
								{syncState.syncing ? 'Syncing...' : 'Sync Now'}
							</button>
						</div>
					</div>
				</div>
			{/if}
		</div>
	</div>

	<!-- Create a Little bar that animates up when a sync is running -->
	{#if syncState.show}
		<div
			class={`fixed bottom-0 left-0 right-0  text-white p-4
			${syncState.syncing ? 'bg-blue-500' : syncState.success ? 'bg-green-500' : 'bg-red-500'}
		
		`}
			transition:slide={{ duration: 500 }}
		>
			<div class="flex items-center">
				{#if !syncState.success && !syncState.error}
					<!-- Add loading spinner -->
					<svg
						xmlns="http://www.w3.org/2000/svg"
						fill="none"
						viewBox="0 0 24 24"
						stroke-width="1.5"
						stroke="currentColor"
						class="size-6 mr-2 animate-spin"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182m0-4.991v4.99"
						/>
					</svg>

					<!-- Add Syncing title and status subtitle below it -->
					<div class="flex flex-col">
						<h1 class="text-lg font-bold">Syncing...</h1>
						<p class="text-sm">
							Please wait while we sync your followers and subscriber data. This can take awhile on
							some connections.
						</p>
					</div>
				{:else if syncState.success}
					<!-- Add Success title and status subtitle below it -->
					<div class="flex flex-col">
						<h1 class="text-lg font-bold">Sync Successful!</h1>
						<p class="text-sm">
							Data synced successfully. Please run the import with the following link {syncState.url}
							to import the data.
						</p>
					</div>

					<!-- Add copy and close button -->
					<div class="flex items-center ml-auto">
						<button
							class="bg-white text-blue-600 px-2 py-1 rounded-lg"
							on:click={() => {
								writeText(syncState.url);
							}}
						>
							Copy
						</button>
						<button
							class="bg-white text-blue-600 px-2 py-1 rounded-lg ml-2"
							on:click={() => {
								syncState.show = false;
							}}
						>
							Close
						</button>
					</div>
				{:else}
					<!-- Add Error title and status subtitle below it -->
					<div class="flex flex-col">
						<h1 class="text-lg font-bold">Sync Failed!</h1>
						<p class="text-sm">
							An error occurred while syncing your data. Details: {syncState.message}
						</p>
					</div>

					<!-- Add close button -->
					<button
						class="bg-white text-blue-600 px-2 py-1 rounded-lg ml-auto"
						on:click={() => {
							syncState.show = false;
						}}
					>
						Close
					</button>
				{/if}
			</div>
		</div>
	{/if}
</div>
