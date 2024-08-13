export type Config = {
	version: number;
	is_first_run: boolean;
	fansly_token: string;
	auto_sync_enabled: boolean;
	sync_token: string;
	sync_interval: number;
	last_sync: number;
	last_sync_data: SyncData;
};

export interface SyncData {
	followers: Follower[];
	subscribers: Subscriber[];
	sync_data_url: string;
}

interface Subscriber {
	id: string;
	historyId: string;
	subscriberId: string;
	subscriptionTierId: string;
	subscriptionTierName: string;
	subscriptionTierColor: string;
	planId: string;
	promoId: null | string;
	giftCodeId: null | string;
	paymentMethodId: string;
	status: number;
	price: number;
	renewPrice: number;
	renewCorrelationId: string;
	autoRenew: number;
	billingCycle: number;
	duration: number;
	renewDate: number;
	version: number;
	createdAt: number;
	updatedAt: number;
	endsAt: number;
	promoPrice: null | number;
	promoDuration: null | number;
	promoStatus: null | number;
	promoStartsAt: null | number;
	promoEndsAt: null | number;
}

interface Follower {
	followerId: string;
}

export interface AccountInfoResponse {
	success: boolean;
	response: AccountInfo[];
}

export interface AccountInfo {
	id: string;
	username: string;
	displayName: string;
	flags: number;
	version: number;
	createdAt: number;
	followCount: number;
	subscriberCount: number;
	permissions: Permissions;
	profileAccessFlags: number;
	profileFlags: number;
	about: string;
	location: string;
	profileSocials: ProfileSocial[];
	pinnedPosts: PinnedPost[];
	walls: Wall[];
	timelineStats: TimelineStats;
	statusId: number;
	lastSeenAt: number;
	mediaStoryState: MediaStoryState;
	accountMediaLikes: number;
	avatar: Avatar;
	banner: Avatar;
	postLikes: number;
	streaming: Streaming;
	subscriptionTiers: SubscriptionTier[];
	profileAccess: boolean;
}

export interface SubscriptionTier {
	id: string;
	accountId: string;
	name: string;
	color: string;
	pos: number;
	price: number;
	maxSubscribers: number;
	subscriptionBenefits: string[];
	includedTierIds: string[];
	plans: Plan[];
}

interface Plan {
	id: string;
	status: number;
	billingCycle: number;
	price: number;
	useAmounts: number;
	promos: Promo[];
	uses: number;
}

interface Promo {
	id: string;
	status: number;
	price: number;
	duration: number;
	maxUses: number;
	maxUsesBefore?: unknown;
	newSubscribersOnly: number;
	startsAt: number;
	endsAt: number;
	uses: number;
}

interface Streaming {
	accountId: string;
	channel: Channel;
	enabled: boolean;
}

interface Channel {
	id: string;
	accountId: string;
	playbackUrl: string;
	chatRoomId: string;
	status: number;
	version: number;
	createdAt: number;
	updatedAt?: unknown;
	stream: Stream;
	arn?: unknown;
	ingestEndpoint?: unknown;
}

interface Stream {
	id: string;
	historyId: string;
	channelId: string;
	accountId: string;
	title: string;
	status: number;
	viewerCount: number;
	version: number;
	createdAt: number;
	updatedAt?: unknown;
	lastFetchedAt: number;
	startedAt: number;
	permissions: Permissions2;
}

interface Permissions2 {
	permissionFlags: PermissionFlag[];
}

interface PermissionFlag {
	id: string;
	streamId: string;
	type: number;
	flags: number;
	price: number;
	metadata: string;
}

interface Avatar {
	id: string;
	type: number;
	status: number;
	accountId: string;
	mimetype: string;
	flags: number;
	location: string;
	width: number;
	height: number;
	metadata: string;
	updatedAt: number;
	createdAt: number;
	variants: Variant[];
	variantHash: VariantHash;
	locations: Location[];
}

type VariantHash = unknown;

interface Variant {
	id: string;
	type: number;
	status: number;
	mimetype: string;
	flags: number;
	location: string;
	width: number;
	height: number;
	metadata: string;
	updatedAt: number;
	locations: Location[];
}

interface Location {
	locationId: string;
	location: string;
}

interface MediaStoryState {
	accountId: string;
	status: number;
	storyCount: number;
	version: number;
	createdAt: number;
	updatedAt: number;
	hasActiveStories: boolean;
}

interface TimelineStats {
	accountId: string;
	imageCount: number;
	videoCount: number;
	bundleCount: number;
	bundleImageCount: number;
	bundleVideoCount: number;
	fetchedAt: number;
}

interface Wall {
	id: string;
	accountId: string;
	pos: number;
	name: string;
	description: string;
	metadata: string;
}

interface PinnedPost {
	postId: string;
	accountId: string;
	pos: number;
	createdAt: number;
}

interface ProfileSocial {
	providerId: string;
	handle: string;
}

interface Permissions {
	accountPermissionFlags: AccountPermissionFlags;
}

interface AccountPermissionFlags {
	flags: number;
}
