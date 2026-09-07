/**
 * Hub Component
 *
 * The empty-chat landing screen. Visually it's "Pair with no messages yet" —
 * a large time + greeting above a centered, narrower ChatInput. Submitting
 * creates a session and navigates to /pair so the rest of the chat lifecycle
 * lives there.
 */

import { useCallback, useEffect, useMemo, useRef, useState, type RefObject } from 'react';
import { defineMessages, useIntl } from '../i18n';
import { AppEvents } from '../constants/events';
import ChatInput from './ChatInput';
import { ChatInputCard } from './ChatInputCard';
import { ChatState } from '../types/chatState';
import 'react-toastify/dist/ReactToastify.css';
import { View, ViewOptions } from '../utils/navigationUtils';
import { useConfig } from './ConfigContext';
import { getEffectiveWorkingDir, getInitialWorkingDir } from '../utils/workingDir';
import { createSession } from '../sessions';
import LoadingGoose from './LoadingGoose';
import { UserInput } from '../types/message';
import {
  createNextChatExtensionDraft,
  selectNextChatExtensions,
  type NextChatExtensionDraft,
} from '../utils/nextChatExtensions';
import { formatAcpError } from '../acp/errors';
import { toastError } from '../toasts';
import { formatClockDisplay } from '../utils/timeUtils';
import { FileText, Files, PenLine, Search, Sparkles, Table2, type LucideIcon } from 'lucide-react';
import { YIRUO_BRAND } from '../brand';

const i18n = defineMessages({
  goodMorning: { id: 'hub.goodMorning', defaultMessage: 'Good morning' },
  goodAfternoon: { id: 'hub.goodAfternoon', defaultMessage: 'Good afternoon' },
  goodEvening: { id: 'hub.goodEvening', defaultMessage: 'Good evening' },
  brandDescription: {
    id: 'hub.brandDescription',
    defaultMessage: 'A calm workspace for documents, writing, research, and daily work.',
  },
  chooseTask: { id: 'hub.chooseTask', defaultMessage: 'Start with a task' },
  documentProcessing: { id: 'hub.documentProcessing', defaultMessage: 'Document processing' },
  documentProcessingPrompt: {
    id: 'hub.documentProcessingPrompt',
    defaultMessage: 'Please help me analyze and organize this document.',
  },
  officialWriting: { id: 'hub.officialWriting', defaultMessage: 'Official writing' },
  officialWritingPrompt: {
    id: 'hub.officialWritingPrompt',
    defaultMessage: 'Please help me turn these materials into a formal work report.',
  },
  webResearch: { id: 'hub.webResearch', defaultMessage: 'Web research' },
  webResearchPrompt: {
    id: 'hub.webResearchPrompt',
    defaultMessage: 'Please research this topic and include reliable sources.',
  },
  compareFiles: { id: 'hub.compareFiles', defaultMessage: 'Compare files' },
  compareFilesPrompt: {
    id: 'hub.compareFilesPrompt',
    defaultMessage: 'Please compare these files and highlight the important differences.',
  },
  dataAnalysis: { id: 'hub.dataAnalysis', defaultMessage: 'Data analysis' },
  dataAnalysisPrompt: {
    id: 'hub.dataAnalysisPrompt',
    defaultMessage: 'Please analyze this data and summarize the key findings.',
  },
  knowledgeBase: { id: 'hub.knowledgeBase', defaultMessage: 'My knowledge base' },
  knowledgeBasePrompt: {
    id: 'hub.knowledgeBasePrompt',
    defaultMessage: 'Please find the most relevant information in my local materials.',
  },
});

type StarterTask = {
  icon: LucideIcon;
  label: keyof typeof i18n;
  prompt: keyof typeof i18n;
};

const STARTER_TASKS: StarterTask[] = [
  { icon: FileText, label: 'documentProcessing', prompt: 'documentProcessingPrompt' },
  { icon: PenLine, label: 'officialWriting', prompt: 'officialWritingPrompt' },
  { icon: Search, label: 'webResearch', prompt: 'webResearchPrompt' },
  { icon: Files, label: 'compareFiles', prompt: 'compareFilesPrompt' },
  { icon: Table2, label: 'dataAnalysis', prompt: 'dataAnalysisPrompt' },
  { icon: Sparkles, label: 'knowledgeBase', prompt: 'knowledgeBasePrompt' },
];

function useClock() {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    const interval = setInterval(() => setNow(new Date()), 30_000);
    return () => clearInterval(interval);
  }, []);

  return formatClockDisplay(now);
}

export default function Hub({
  setView,
  draftRef,
}: {
  setView: (view: View, viewOptions?: ViewOptions) => void;
  /** Unsent input of this screen, kept above the route outlet across the unmount. */
  draftRef: RefObject<string>;
}) {
  const intl = useIntl();
  const { extensionsList } = useConfig();
  const [workingDir, setWorkingDir] = useState(getInitialWorkingDir());
  const userSelectedWorkingDirRef = useRef(false);
  const [isCreatingSession, setIsCreatingSession] = useState(false);
  const [nextChatExtensionDraft, setNextChatExtensionDraft] =
    useState<NextChatExtensionDraft | null>(null);
  const [starterPrompt, setStarterPrompt] = useState('');
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const { time, meridiem, hour } = useClock();

  // Re-resolve the working dir on mount: GOOSE_WORKING_DIR is fixed at window
  // creation, so a configured remote directory may have changed since then.
  useEffect(() => {
    let active = true;
    void getEffectiveWorkingDir().then((dir) => {
      if (active && !userSelectedWorkingDirRef.current) setWorkingDir(dir);
    });
    return () => {
      active = false;
    };
  }, []);

  const greeting = useMemo(() => {
    if (hour < 12) return intl.formatMessage(i18n.goodMorning);
    if (hour < 18) return intl.formatMessage(i18n.goodAfternoon);
    return intl.formatMessage(i18n.goodEvening);
  }, [intl, hour]);

  const draftForMenu = useMemo(
    () => nextChatExtensionDraft ?? createNextChatExtensionDraft(extensionsList),
    [extensionsList, nextChatExtensionDraft]
  );

  // rAF is more reliable than autoFocus across async render boundaries.
  useEffect(() => {
    const frameId = requestAnimationFrame(() => {
      inputRef.current?.focus();
    });
    return () => cancelAnimationFrame(frameId);
  }, []);

  const handleNextChatExtensionDraftChange = useCallback((draft: NextChatExtensionDraft) => {
    setNextChatExtensionDraft(draft);
  }, []);

  const handleWorkingDirChange = useCallback((dir: string) => {
    userSelectedWorkingDirRef.current = true;
    setWorkingDir(dir);
  }, []);

  const handleStarterTask = useCallback(
    (prompt: keyof typeof i18n) => {
      setStarterPrompt(intl.formatMessage(i18n[prompt]));
      requestAnimationFrame(() => inputRef.current?.focus());
    },
    [intl]
  );

  const handleSubmit = async (input: UserInput) => {
    const { msg: userMessage, images } = input;
    if (!(images.length > 0 || userMessage.trim()) || isCreatingSession) return;

    const draftAtSubmit = draftRef.current;
    setIsCreatingSession(true);

    try {
      const selectedExtensions = nextChatExtensionDraft
        ? selectNextChatExtensions(extensionsList, nextChatExtensionDraft)
        : [];
      const sessionOptions =
        selectedExtensions.length > 0
          ? { extensionConfigs: selectedExtensions }
          : { allExtensions: extensionsList };

      // Resolve the effective directory at submit time: the IPC lookup may still
      // be pending when the user submits, and an explicit pick must win.
      const dir = userSelectedWorkingDirRef.current ? workingDir : await getEffectiveWorkingDir();
      const session = await createSession(dir, sessionOptions);
      setNextChatExtensionDraft(null);

      window.dispatchEvent(new CustomEvent(AppEvents.SESSION_CREATED));
      window.dispatchEvent(
        new CustomEvent(AppEvents.ADD_ACTIVE_SESSION, {
          detail: { sessionId: session.id, initialMessage: { msg: userMessage, images } },
        })
      );

      // The draft is this screen's own, so it is dropped once the session exists.
      // Comparing it against the value at submit leaves an edit made while the
      // session was starting alone, including one that emptied the input.
      if (draftRef.current === draftAtSubmit) {
        draftRef.current = '';
      }

      setView('pair', {
        disableAnimation: true,
        resumeSessionId: session.id,
        initialMessage: { msg: userMessage, images },
      });
    } catch (error) {
      console.error('Failed to create session:', error);
      toastError({ title: "Couldn't start chat", msg: formatAcpError(error) });
      setIsCreatingSession(false);
    }
  };

  return (
    <div className="flex flex-col h-full min-h-0 items-center justify-center px-6 py-10 relative overflow-y-auto">
      <div className="w-full max-w-3xl">
        <div className="mb-10">
          <div className="flex items-center gap-3 mb-3">
            <div className="flex size-10 items-center justify-center rounded-xl bg-background-inverse text-text-inverse">
              <Sparkles className="size-5" aria-hidden="true" />
            </div>
            <div>
              <div className="text-2xl font-medium tracking-tight text-text-primary">
                {YIRUO_BRAND.name}
              </div>
              <div className="text-xs text-text-secondary">{YIRUO_BRAND.slogan}</div>
            </div>
          </div>
          <p className="max-w-xl text-sm text-text-secondary">
            {intl.formatMessage(i18n.brandDescription)}
          </p>
        </div>

        <div className="flex items-baseline gap-2 mb-1">
          <span className="text-6xl font-light text-text-primary tracking-tight tabular-nums">
            {time}
          </span>
          {meridiem ? (
            <span className="text-2xl font-light text-text-secondary">{meridiem}</span>
          ) : null}
        </div>
        <p className="text-xl text-text-secondary mb-6">{greeting}</p>

        <div className="mb-3 text-sm font-medium text-text-primary">
          {intl.formatMessage(i18n.chooseTask)}
        </div>
        <div className="grid grid-cols-2 gap-2 sm:grid-cols-3 mb-6">
          {STARTER_TASKS.map(({ icon: Icon, label, prompt }) => (
            <button
              key={label}
              type="button"
              onClick={() => handleStarterTask(prompt)}
              className="flex min-h-20 items-center gap-3 rounded-lg border border-border-primary bg-background-primary px-3 text-left text-sm text-text-primary transition-colors hover:bg-background-secondary focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
            >
              <Icon className="size-5 shrink-0 text-text-secondary" aria-hidden="true" />
              <span>{intl.formatMessage(i18n[label])}</span>
            </button>
          ))}
        </div>

        <ChatInputCard>
          <ChatInput
            sessionId={null}
            draftRef={draftRef}
            handleSubmit={handleSubmit}
            chatState={isCreatingSession ? ChatState.LoadingConversation : ChatState.Idle}
            onStop={() => {}}
            initialValue={starterPrompt}
            setView={setView}
            totalTokens={0}
            accumulatedInputTokens={0}
            accumulatedOutputTokens={0}
            droppedFiles={[]}
            onFilesProcessed={() => {}}
            messages={[]}
            disableAnimation={false}
            workingDir={workingDir}
            onWorkingDirChange={handleWorkingDirChange}
            inputRef={inputRef}
            nextChatExtensionDraft={draftForMenu}
            onNextChatExtensionDraftChange={handleNextChatExtensionDraftChange}
          />
        </ChatInputCard>
      </div>

      {isCreatingSession && (
        <div className="absolute bottom-4 left-4 z-20 pointer-events-none">
          <LoadingGoose chatState={ChatState.LoadingConversation} />
        </div>
      )}
    </div>
  );
}
