# FanslySync Developer Documentation

Welcome. We'll tell you how to integrate with FanslySync.

## Our JSON Schema

FanslySync uses a JSON schema to sync data with 3rd party services. Here's an example of the JSON schema:

```json
{
	"followers": [{ "followerId": "123456" }],
	"subscribers": [
		// An array of subscriber objects. See below for the schema.
	]
}
```

### Subscriber Schema

```json
{
  {
      "id": "0", // The ID of the subscription, usually unique
      "historyId": "<history_id>", // The ID of the subscription history
      "subscriberId": "<user_id>", // The User ID of the subscriber
      "subscriptionTierId": "<tier_id>", // The ID of the subscription tier
      "subscriptionTierName": "<tier_name>", // The name of the subscription tier
      "subscriptionTierColor": "#2699f7", // The color of the subscription tier
      "planId": "0", // The ID of the subscription plan
      "promoId": "0", // The ID of the promotion, if applicable
      "giftCodeId": null, // The ID of the gift code, if applicable
      "paymentMethodId": "0", // The ID of the payment method
      "status": 3, // The status of the subscription. 3 = active, 4 = ?
      "price": 7000, // The price of the subscription, in cents
      "renewPrice": 7000, // The price of the subscription renewal, in cents
      "renewCorrelationId": "673162822363914240", // The correlation ID of the renewal
      "autoRenew": 1, // Whether the subscription is set to auto-renew
      "billingCycle": 30, // The billing cycle of the subscription, in days
      "duration": 30, // The duration of the subscription, in days
      "renewDate": 1721988883000, // The date the subscription will renew (UNIX timestamp)
      "version": 3, // The version of the subscription schema from fansly
      "createdAt": 1721988883000, // The date the subscription was created (UNIX timestamp)
      "updatedAt": 1721988883000, // The date the subscription was last updated (UNIX timestamp)
      "endsAt": 1724667283000, // The date the subscription will end (UNIX timestamp)
      "promoPrice": null, // The price of the subscription with the promotion, in cents
      "promoDuration": null, // The duration of the subscription with the promotion, in days
      "promoStatus": null, // The status of the promotion
      "promoStartsAt": null, // The date the promotion starts (UNIX timestamp)
      "promoEndsAt": null // The date the promotion ends (UNIX timestamp)
    },
}
```

# Closing

That's it! If you have any questions, feel free to reach out to us at our [support email](mailto:tanner@fanslycreatorbot.com) if you have any questions. We're happy to help you integrate with FanslySync.
