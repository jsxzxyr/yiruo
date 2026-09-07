import { useMemo } from 'react';
import CardContainer from './CardContainer';
import CardHeader from './CardHeader';
import CardBody from './CardBody';
import DefaultCardButtons from './buttons/DefaultCardButtons';
import type { ProviderDetails, ProviderMetadata } from '../../../../types/providers';
import { defineMessages, useIntl } from '../../../../i18n';
import { getProviderRegistration } from '../../../../providers/providerRegistry';

const i18n = defineMessages({
  noMetadata: {
    id: 'providerCard.noMetadata',
    defaultMessage: 'ProviderCard error: No metadata provided',
  },
  unknownProvider: {
    id: 'providerCard.unknownProvider',
    defaultMessage: 'Unknown Provider',
  },
  deprecatedReplacement: {
    id: 'providerCard.deprecatedReplacement',
    defaultMessage: 'Deprecated — use {replacement} instead.',
  },
  local: {
    id: 'providerCard.local',
    defaultMessage: 'Local',
  },
  external: {
    id: 'providerCard.external',
    defaultMessage: 'External',
  },
});

type ProviderCardProps = {
  provider: ProviderDetails;
  onConfigure: () => void;
  onLaunch: () => void;
  isOnboarding: boolean;
};

export const ProviderCard = function ProviderCard({
  provider,
  onConfigure,
  onLaunch,
  isOnboarding,
}: ProviderCardProps) {
  const intl = useIntl();
  // Safely access metadata with null checks
  const providerMetadata: ProviderMetadata | null = provider?.metadata || null;

  // Instead of useEffect for logging, use useMemo to memoize the metadata
  const metadata = useMemo(() => providerMetadata, [providerMetadata]);

  if (!metadata) {
    return <div>{intl.formatMessage(i18n.noMetadata)}</div>;
  }

  const handleCardClick = () => {
    if (!isOnboarding) {
      onConfigure();
    }
  };
  const description = provider.deprecated
    ? `${metadata.description} ${intl.formatMessage(i18n.deprecatedReplacement, {
        replacement: provider.replacement ?? intl.formatMessage(i18n.unknownProvider),
      })}`
    : metadata.description;
  const registration = getProviderRegistration(provider);

  return (
    <CardContainer
      testId={`provider-card-${provider.name.toLowerCase()}`}
      grayedOut={!provider.is_configured && isOnboarding} // onboarding page will have grayed out cards if not configured
      onClick={handleCardClick}
      header={
        <>
          <CardHeader
            name={
              metadata.display_name || provider?.name || intl.formatMessage(i18n.unknownProvider)
            }
            description={description}
            isConfigured={provider?.is_configured || false}
          />
          <span className="mt-2 inline-flex w-fit rounded-md bg-background-secondary px-1.5 py-0.5 text-[10px] text-text-secondary">
            {intl.formatMessage(registration.dataBoundary === 'local' ? i18n.local : i18n.external)}
          </span>
        </>
      }
      body={
        <CardBody>
          <DefaultCardButtons
            provider={provider}
            onConfigure={onConfigure}
            onLaunch={onLaunch}
            isOnboardingPage={isOnboarding}
          />
        </CardBody>
      }
    />
  );
};
