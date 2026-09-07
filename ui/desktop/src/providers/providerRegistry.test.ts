import { describe, expect, it } from 'vitest';
import { getProviderRegistration } from './providerRegistry';

const provider = (name: string) => ({
  name,
  metadata: { display_name: name },
});

describe('Yiruo provider registry', () => {
  it('marks explicitly local runtimes as local', () => {
    expect(getProviderRegistration(provider('ollama'))).toMatchObject({
      dataBoundary: 'local',
      credentialsStoredByBackend: true,
    });
  });

  it('treats unknown and custom providers as external by default', () => {
    expect(getProviderRegistration(provider('custom_internal_api')).dataBoundary).toBe('external');
  });
});
