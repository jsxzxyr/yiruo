/**
 * Presentation-level provider registration used by Yiruo's settings UI.
 *
 * This is deliberately not a security decision. Phase 3's Policy Engine must
 * make the authoritative data-transfer decision before a model request.
 */
export type ProviderDataBoundary = 'local' | 'external';

export type YiruoProviderRegistration = {
  providerId: string;
  displayName: string;
  dataBoundary: ProviderDataBoundary;
  credentialsStoredByBackend: true;
};

type ProviderRegistrationInput = {
  name: string;
  metadata: { display_name: string };
};

const LOCAL_PROVIDER_IDS = new Set(['ollama', 'vllm', 'llamacpp', 'llama.cpp']);

export function getProviderRegistration(
  provider: ProviderRegistrationInput
): YiruoProviderRegistration {
  const providerId = provider.name.trim();
  const normalizedId = providerId.toLowerCase();

  return {
    providerId,
    displayName: provider.metadata.display_name || providerId,
    dataBoundary: LOCAL_PROVIDER_IDS.has(normalizedId) ? 'local' : 'external',
    credentialsStoredByBackend: true,
  };
}
