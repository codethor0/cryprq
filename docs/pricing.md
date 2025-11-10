# Pricing & Tier Strategy

CrypRQ is open-source and intended to remain freely available for self-hosting. This plan provides optional pricing guidance for store-distributed builds and donation flows.

## Principles

- Keep paid tiers modest (USD $0.49–$0.99 equivalents) to encourage adoption while covering distribution costs.
- Offer optional donation links for F-Droid and Windows sideload builds.
- Maintain “free during beta” until control-plane + data-plane reach GA.
- Communicate clearly when switching from free to paid tiers.

## Recommended Price Tiers

| Region | Suggested Tier | Approx. Local Price |
|--------|----------------|---------------------|
| United States | Tier USD 0.99 | $0.99 |
| Canada | Tier CAD 0.99 | CA$0.99 |
| Eurozone | Tier EUR 0.99 | €0.99 |
| United Kingdom | Tier GBP 0.79 | £0.79 |
| Australia | Tier AUD 1.49 | AU$1.49 |
| Japan | Tier JPY 100 | ¥100 |
| India | Tier INR 75 | ₹75 |

- Align tiers with Apple/Google/Microsoft pricing matrices (`Pricing Tier 1` on Apple, `Tier 3` on Play, etc.).
- F-Droid builds stay free; donation link only.

## Launch Phases

1. **Beta (current):** All stores set to “Free.” Messaging emphasises experimental status.
2. **GA (control-plane stable + data-plane ready):** Switch to paid tiers in Microsoft Store and App Store; Play Store optional free tier with in-app donation link.
3. **Enterprise licensing (future):** Provide contact for bulk/custom support; separate from store pricing.

## Donation Messaging

F-Droid & Windows README snippet:
> CrypRQ remains free and open-source. If you’d like to support ongoing post-quantum security research, consider donating at https://cryprq.dev/donate (PayPal/Stripe/crypto).

## Store Price Change Playbook

1. Announce planned pricing change via release notes and documentation.
2. Update pricing in Apple App Store Connect, Microsoft Partner Center, and Google Play Console.
3. Sync new pricing into `docs/pricing.md` and `fastlane` metadata.
4. Monitor user feedback; consider introductory pricing discounts for the first GA month.

## Compliance Notes

- Apple: if switching from free to paid, ensure contractual agreements remain current and provide “pricing change notice” in `App Review Notes`.
- Google Play: consider `In-app Products` for donation tiers instead of paid APK (can remain free).
- Microsoft Store: support free trial if desired via `StoreListing > Trial` configuration.

## Next Steps

- Keep current builds free until GA readiness review.
- Set up donation infrastructure (Stripe/PayPal) and publish `donate` page.
- Document internal checklist for price change approvals.

