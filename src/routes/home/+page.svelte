<script lang="ts">
	import { info, error } from '@tauri-apps/plugin-log';
	import { awaiter } from '$lib/utils';
	import { onDestroy, onMount } from 'svelte';
	import type { Config, SyncData } from '$lib/types';
	import { fade, fly, slide } from 'svelte/transition';
	import { sendNotification } from '@tauri-apps/plugin-notification';
	import { platform } from '@tauri-apps/plugin-os';
	import { check, Update } from '@tauri-apps/plugin-updater';
	import { getVersion, getTauriVersion } from '@tauri-apps/api/app';
	import { invoke } from '@tauri-apps/api/core';
	import { ask, message } from '@tauri-apps/plugin-dialog';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { isEnabled, enable } from '@tauri-apps/plugin-autostart';
	import { toast } from 'svelte-french-toast';

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

	let isAutoSyncConfigModalOpen = false;
	let canSave = false;
	let config: Config | null = null;
	let syncInterval: number | null = null;

	let autoSyncConfig = {
		interval: 0,
		syncToken: '',
		didRunInitialValidation: false
	};

	let autoSyncConfigState = {
		validatingToken: false,
		tokenValid: false
	};

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
		autoSyncConfig.interval = config.sync_interval;
		autoSyncConfig.syncToken = config.sync_token;
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

		info(`[FanslySync::page_init:home] Setting up autosync interval...`);
		syncInterval = setInterval(
			async () => {
				if (config?.auto_sync_enabled) {
					info(`[FanslySync::autoSyncProcess] Auto Sync enabled - syncing data automatically.`);
					const nextIntervalTime = new Date(
						Date.now() + (config?.sync_interval ?? 1) * 60 * 60 * 1000
					);
					const nextIntervalTimeString = nextIntervalTime.toLocaleTimeString();
					const returnedData = await syncNow(true);
					if (!returnedData || returnedData === null) {
						error(`[FanslySync::autoSyncProcess] Failed to sync data automatically.`);

						// Send error notification
						await sendNotification({
							title: 'FanslySync: Auto Sync Failed!',
							body: `An error occurred while syncing data automatically. We will automatically retry at ${nextIntervalTimeString}.`
						});
					} else {
						info(
							`[FanslySync::autoSyncProcess] Synced data automatically - preparing to send to server.`
						);

						const [_, uploadErr] = await awaiter(
							invoke('fansly_upload_auto_sync_data', {
								token: config?.sync_token,
								data: returnedData
							})
						);

						if (uploadErr) {
							error(
								`[FanslySync::autoSyncProcess] Failed to upload data to server. Error: ${uploadErr}`
							);

							// Send error notification
							await sendNotification({
								title: 'FanslySync: Auto Sync Failed!',
								body: `An error occurred while uploading data to the server. We will automatically retry at ${nextIntervalTimeString}.`
							});

							return;
						}

						// Send success notification
						await sendNotification({
							title: 'FanslySync: Auto Sync Successful!',
							body: `Data synced and uploaded successfully. Next sync will occur at ${nextIntervalTimeString}.`
						});
					}
				} else {
					info(`[FanslySync::autoSyncProcess] Auto Sync disabled - skipping this sync.`);
				}
			},
			(config?.sync_interval ?? 1) * 60 * 60 * 1000
		);

		info(`[FanslySync::page_init:home] Autosync interval set successfully.`);
		info(`[FanslySync::page_init:home] Page initialization completed successfully.`);
	});

	async function syncNow(auto: boolean = false) {
		info(`[FanslySync::syncNow] Starting manual sync...`);

		syncState.error = false;
		syncState.success = false;
		syncState.syncing = true;
		syncState.show = !auto;

		const [syncData, syncError] = await awaiter(
			invoke('fansly_sync', {
				auto
			}) as Promise<SyncData>
		);

		if (syncError || syncData === null) {
			error(
				`[FanslySync::syncNow] Failed to sync data. Error: ${syncError ?? 'Sync data was null'}`
			);
			syncState.syncing = false;
			syncState.error = true;
			syncState.message = syncError ?? 'Sync data was null';

			// Send failure notification
			await sendNotification({
				title: 'FanslySync: Sync Failed!',
				body: 'An error occurred while syncing data. Please check the app for more details.'
			});

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

			// Send failure notification
			await sendNotification({
				title: 'FanslySync: Sync Failed!',
				body: 'An error occurred while syncing data. Please check the app for more details.'
			});
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

		if (!auto)
			await sendNotification({
				title: 'FanslySync: Sync Successful!',
				body: 'Data synced successfully. Please look at the app for more details.',
				sound: soundName
			});

		info(`[FanslySync::syncNow] Manual sync completed successfully.`);

		if (auto) return syncData;
		else return null;
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

	async function enableAutoSync() {
		const isAutoStartEnabled = await isEnabled();

		if (!isAutoStartEnabled) {
			// Required to enable autosync. Ask user for permission
			const confirm = await ask(
				`We've detected that FanslySync is not set to start automatically with the system. To enable Auto Sync, we need to enable this feature. Would you like to enable this feature now?`,
				{
					title: 'FanslySync | Enable Auto Start',
					okLabel: 'Yes',
					cancelLabel: 'No',
					kind: 'warning'
				}
			);

			if (!confirm) {
				// Error out
				await message(
					`Auto Sync cannot be enabled without enabling the Auto Start feature. Please enable the Auto Start feature and try again.`,
					{ title: 'FanslySync | Auto Sync Error', kind: 'error' }
				);

				return;
			} else {
				await toast.promise(enable(), {
					loading: 'Enabling Auto Start...',
					success: 'Auto Start enabled successfully.',
					error: 'Failed to enable Auto Start. Please try again.'
				});
			}
		}

		// Ensure they have a sync token set
		if (!config?.sync_token || config?.sync_token.length === 0) {
			await message(
				`Auto Sync cannot be enabled without a valid sync token. Please set a sync token in settings and try again.`,
				{ title: 'FanslySync | Auto Sync Error', kind: 'error' }
			);

			return;
		}

		// Enable autosync
		config!.auto_sync_enabled = !config?.auto_sync_enabled;
		const [_, saveConfigError] = await awaiter(invoke('save_config', { config }));

		if (saveConfigError) {
			await message(
				`Failed to save Auto Sync configuration. Please try again. Error: ${saveConfigError}`,
				{ title: 'FanslySync | Auto Sync Error', kind: 'error' }
			);
			error(
				`[FanslySync::enableAutoSync] Failed to save Auto Sync configuration. Error: ${saveConfigError}`
			);
			return;
		}

		// Clear interval if autosync is disabled, set if enabled
		if (config?.auto_sync_enabled) {
			syncInterval = setInterval(
				async () => {
					if (config?.auto_sync_enabled) {
						info(`[FanslySync::autoSyncProcess] Auto Sync enabled - syncing data automatically.`);
						const nextIntervalTime = new Date(
							Date.now() + (config?.sync_interval ?? 1) * 60 * 60 * 1000
						);
						const nextIntervalTimeString = nextIntervalTime.toLocaleTimeString();
						const returnedData = await syncNow(true);
						if (!returnedData || returnedData === null) {
							error(`[FanslySync::autoSyncProcess] Failed to sync data automatically.`);

							// Send error notification
							await sendNotification({
								title: 'FanslySync: Auto Sync Failed!',
								body: `An error occurred while syncing data automatically. We will automatically retry at ${nextIntervalTimeString}.`
							});
						} else {
							info(
								`[FanslySync::autoSyncProcess] Synced data automatically - preparing to send to server.`
							);

							const [_, uploadErr] = await awaiter(
								invoke('fansly_upload_auto_sync_data', {
									token: config?.sync_token,
									data: returnedData
								})
							);

							if (uploadErr) {
								error(
									`[FanslySync::autoSyncProcess] Failed to upload data to server. Error: ${uploadErr}`
								);

								// Send error notification
								await sendNotification({
									title: 'FanslySync: Auto Sync Failed!',
									body: `An error occurred while uploading data to the server. We will automatically retry at ${nextIntervalTimeString}.`
								});

								return;
							}

							// Send success notification
							await sendNotification({
								title: 'FanslySync: Auto Sync Successful!',
								body: `Data synced and uploaded successfully. Next sync will occur at ${nextIntervalTimeString}.`
							});
						}
					} else {
						info(`[FanslySync::autoSyncProcess] Auto Sync disabled - skipping this sync.`);
					}
				},
				(config?.sync_interval ?? 1) * 60 * 60 * 1000
			);

			// Run a auto sync as soon as it's enabled
			const returnedData = await syncNow(true);
			if (!returnedData || returnedData === null) {
				error(`[FanslySync::autoSyncProcess] Failed to sync data automatically.`);
				// Disable autosync, resave, and error out on the UI
				config!.auto_sync_enabled = false;
				const [_, saveConfigError] = await awaiter(invoke('save_config', { config }));

				if (saveConfigError)
					toast.error('Failed to save Auto Sync configuration. Please try again.', {
						duration: 5000
					});

				toast.error(
					'Self test of Auto Sync failed (SYNC_ERR). Please check the logs for more details.',
					{
						duration: 5000
					}
				);

				return;
			} else {
				info(
					`[FanslySync::autoSyncProcess] Synced data automatically - preparing to send to server.`
				);

				const [_, uploadErr] = await awaiter(
					invoke('fansly_upload_auto_sync_data', {
						token: config?.sync_token,
						data: returnedData
					})
				);

				if (uploadErr) {
					error(
						`[FanslySync::autoSyncProcess] Failed to upload data to server. Error: ${uploadErr}`
					);

					// Disable autosync, resave, and error out on the UI
					config!.auto_sync_enabled = false;
					const [_, saveConfigError] = await awaiter(invoke('save_config', { config }));
					if (saveConfigError)
						toast.error('Failed to save Auto Sync configuration. Please try again.', {
							duration: 5000
						});

					toast.error(
						'Self test of Auto Sync failed (UPLOAD_ERR). Please check the logs for more details.',
						{
							duration: 5000
						}
					);

					return;
				}
			}
		} else {
			if (syncInterval) {
				clearInterval(syncInterval);
			}
		}

		info(`[FanslySync::enableAutoSync] Auto Sync configuration saved successfully.`);
		const nextInterval = new Date(Date.now() + (config?.sync_interval ?? 1) * 60 * 60 * 1000);
		const nextIntervalString = nextInterval.toLocaleTimeString();

		toast.success(
			`${
				config?.auto_sync_enabled ? 'Enabled' : 'Disabled'
			} Auto Sync successfully. ${config?.auto_sync_enabled ? `The next sync will occur at ${nextIntervalString}.` : ''}`,
			{
				duration: 5000
			}
		);
	}

	async function onSyncTokenEntered() {
		// Check if the sync token is valid
		autoSyncConfigState.validatingToken = true;

		const [_, tokenError] = await awaiter(
			invoke('fansly_check_sync_token', { token: autoSyncConfig.syncToken })
		);

		if (tokenError) {
			error(`[FanslySync::onSyncTokenEntered] Failed to validate sync token. Error: ${tokenError}`);
			autoSyncConfigState.validatingToken = false;
			autoSyncConfigState.tokenValid = false;
			canSave = false;
			return;
		} else {
			info(`[FanslySync::onSyncTokenEntered] Sync token validated successfully.`);
			autoSyncConfigState.validatingToken = false;
			autoSyncConfigState.tokenValid = true;
			canSave = true;
		}
	}

	async function onAutoSyncSave() {
		// Close the modal and reset the didRunInitialValidation flag
		autoSyncConfig.didRunInitialValidation = false;
		isAutoSyncConfigModalOpen = false;

		const savingToast = await toast.loading('Saving Auto Sync configuration...');

		config!.sync_interval = autoSyncConfig.interval;
		config!.sync_token = autoSyncConfig.syncToken;

		const [_, saveConfigError] = await awaiter(invoke('save_config', { config }));

		if (saveConfigError) {
			await toast.error('Failed to save Auto Sync configuration. Please try again.', {
				id: savingToast
			});
			error(
				`[FanslySync::onAutoSyncSave] Failed to save Auto Sync configuration. Error: ${saveConfigError}`
			);
			return;
		} else {
			info(`[FanslySync::onAutoSyncSave] Auto Sync configuration saved successfully.`);
			await toast.success('Auto Sync configuration saved successfully.', { id: savingToast });
		}
	}

	// When the component is destroyed, clear the interval
	onDestroy(() => {
		info(`[FanslySync::page_destroy:home] onDestroy() called. Cleaning up...`);
		if (syncInterval) {
			info(`[FanslySync::page_destroy:home] Clearing autosync interval...`);
			clearInterval(syncInterval);
		}

		info(`[FanslySync::page_destroy:home] Cleaning up completed. Goodbye!`);
	});

	// When we show the modal, run the initial validation
	$: if (isAutoSyncConfigModalOpen && !autoSyncConfig.didRunInitialValidation) {
		onSyncTokenEntered();
		autoSyncConfig.didRunInitialValidation = true;
	}

	function useDebounce(fn: Function, delay: number) {
		let timeout: number;
		return function (...args: any) {
			clearTimeout(timeout);
			timeout = setTimeout(() => fn(...args), delay);
		};
	}
</script>

<div class="container bg-zinc-800 w-screen h-screen">
	<!-- Create config modal -->
	{#if isAutoSyncConfigModalOpen}
		<div
			class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-10"
			transition:fade={{ duration: 300 }}
		>
			<div
				class="bg-zinc-900 p-4 rounded-lg"
				in:fly={{ y: 20, duration: 300 }}
				out:fly={{ y: 20, duration: 200 }}
			>
				<h1 class="text-2xl font-bold text-gray-200">Auto Sync Configuration</h1>
				<p class="text-gray-400 mt-1">
					Configure the Auto Sync feature here. Please ensure you have a valid sync token set in
					settings.
				</p>
				<div class="flex flex-col mt-2">
					<label for="syncInterval" class="text-gray-200">Sync Interval (in hours)</label>
					<input
						type="number"
						id="syncInterval"
						class="bg-zinc-700 text-gray-200 p-2 rounded-lg mt-1"
						placeholder="Enter sync interval in hours"
						bind:value={autoSyncConfig.interval}
						min="1"
					/>
					<p class="text-gray-400 mt-1">
						How often should the app sync data automatically? Please enter a number in hours. The
						minimum value is 1 hour.
					</p>
					<label for="syncToken" class="text-gray-200 mt-2">Sync Token</label>
					<div class="relative flex items-center">
						<!-- Sync token input. Check in the input for valid token, x not, question mark empty -->
						<input
							type="text"
							id="syncToken"
							class="bg-zinc-700 text-gray-200 p-2 rounded-lg mt-1 w-full pr-10"
							placeholder="Enter sync token"
							bind:value={autoSyncConfig.syncToken}
							on:input={useDebounce(onSyncTokenEntered, 500)}
						/>
						{#if !autoSyncConfigState.validatingToken && autoSyncConfigState.tokenValid}
							<svg
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="1.5"
								stroke="currentColor"
								class="absolute right-2 bottom-2 w-6 h-6 text-green-500"
							>
								<path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
							</svg>
						{:else if !autoSyncConfigState.validatingToken && !autoSyncConfigState.tokenValid}
							<svg
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="1.5"
								stroke="currentColor"
								class="absolute right-2 bottom-2 w-6 h-6 text-red-500"
							>
								<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
							</svg>
						{:else if autoSyncConfigState.validatingToken}
							<svg
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="1.5"
								stroke="currentColor"
								class="absolute right-2 bottom-2 w-6 h-6 text-gray-400 animate-spin"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182m0-4.991v4.99"
								/>
							</svg>
						{/if}
					</div>
					<p class="text-gray-400 mt-1">
						Enter your sync token here. A green tick will be displayed if the token is valid, a red
						cross if it's invalid, and a spinner if it's validating. Please ensure you have a valid
						sync token set or automatic sync will not work.
					</p>
					<div class="flex mt-2">
						<button
							class="bg-blue-600 text-white px-4 py-2 rounded-lg w-full disabled:opacity-50 disabled:cursor-not-allowed hover:bg-blue-700 transition-all duration-200 ease-in-out"
							disabled={!canSave}
							on:click={onAutoSyncSave}
						>
							{canSave ? 'Save' : 'Please enter a valid sync token'}
						</button>
						<button
							class="bg-red-500 text-white px-4 py-2 rounded-lg w-full ml-2 disabled:opacity-50 disabled:cursor-not-allowed hover:bg-red-600 transition-all duration-200 ease-in-out"
							on:click={() => {
								isAutoSyncConfigModalOpen = false;
								autoSyncConfig.didRunInitialValidation = false;
							}}
						>
							Cancel
						</button>
					</div>
				</div>
			</div>
		</div>
	{/if}

	<!-- Top header, FanslySync at the left side, settings icon on the right -->
	<div class="flex justify-between items-center h-16 px-4 bg-zinc-900">
		<div class="flex items-center">
			<img src="/fanslySync.png" alt="FanslySync" class="w-8 h-8" />
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
							<div class="flex items-center space-x-2">
								<h1 class="text-xl font-bold text-gray-200">Automatic Sync</h1>
								<!-- Status badge -->
								<span
									class={`px-2 py-1 rounded-lg text-xs font-bold ${
										config?.auto_sync_enabled ? 'bg-green-500' : 'bg-red-500'
									}`}
								>
									{config?.auto_sync_enabled ? 'Enabled' : 'Disabled'}
								</span>
							</div>

							<p class="text-gray-400 mt-1">
								Sync content automatically every {config?.sync_interval}
								{(config?.sync_interval ?? 0 > 1) ? 'hour' : 'hours'}. Please ensure you have a
								stable internet connection.
							</p>
							<div class="flex mt-2">
								<button
									class={` text-white px-4 py-2 rounded-lg w-full ${
										!config?.auto_sync_enabled
											? 'bg-green-500 hover:bg-green-600'
											: 'bg-red-500 hover:bg-red-600'
									} disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 ease-in-out`}
									on:click={enableAutoSync}
									disabled={syncState.syncing}
								>
									{syncState.syncing
										? 'Sync in progress. Please wait.'
										: config?.auto_sync_enabled
											? 'Disable'
											: 'Enable'}
									{!syncState.syncing ? 'Auto Sync' : ''}
								</button>
								<button
									class="bg-blue-600 text-white px-4 py-2 rounded-lg w-full ml-2 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 ease-in-out hover:bg-blue-700"
									on:click={() => {
										isAutoSyncConfigModalOpen = true;
									}}
									disabled={config?.auto_sync_enabled || syncState.syncing}
								>
									{syncState.syncing
										? 'Sync in progress. Please wait.'
										: config?.auto_sync_enabled
											? 'Disable Auto Sync To Edit'
											: 'Edit Auto Sync Configuration'}
								</button>
							</div>
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
								on:click={() => {
									syncNow(false);
								}}
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
