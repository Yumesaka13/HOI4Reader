import { A11yModule } from '@angular/cdk/a11y';
import { BreakpointObserver } from '@angular/cdk/layout';
import { ChangeDetectorRef, Component, HostListener, inject, OnInit, TemplateRef } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { CommonModule } from '@angular/common';
import { projectForeignFocuses } from './foreign-focus-calendar';
import { evaluateCondition, ScriptCondition } from './script-condition';
import { UI_TEXT } from './ui-text';
import { FormsModule } from '@angular/forms';
import { Channel, convertFileSrc, invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { MatSidenavModule } from '@angular/material/sidenav';
import { MatToolbarModule } from '@angular/material/toolbar';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';
import { MatListModule } from '@angular/material/list';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatChipsModule } from '@angular/material/chips';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { MatBottomSheet, MatBottomSheetModule, MatBottomSheetRef } from '@angular/material/bottom-sheet';
import { map } from 'rxjs';

type Page = 'home' | 'mods' | 'countries' | 'country-intro' | 'chapters' | 'focus-choice' | 'reader' | 'settings' | 'music' | 'search';
type Lang = 'english' | 'simp_chinese' | 'traditional_chinese' | 'japanese';
type MusicTransition = 'interrupt' | 'queue';
type ReaderTheme = 'material' | 'tfr' | 'tno';
type ThemePreference = ReaderTheme | 'auto';

interface ModInfo {
  parent_name?: string | null;
  name: string;
  path: string;
  workshop_id: string;
  image_path?: string | null;
}

interface Discovery {
  game_path: string;
  workshop_path: string;
  mods: ModInfo[];
}

interface SuperEventTrack {
  title: string;
  path: string;
}

interface RememberedTrack extends SuperEventTrack {
  mod: string;
}

interface DownloadProgress {
  received: number;
  total: number | null;
  installing: boolean;
}

interface SearchDocument {
  item: Chapter | TimelineItem;
  fields: SearchField[];
}

interface SearchField {
  kind: 'title' | 'description' | 'option';
  raw: string;
  text: string;
  normalized: string;
}

interface SearchResult {
  item: Chapter | TimelineItem;
  excerpt: string;
  match?: SearchField;
}

type MusicEffect = { kind: 'play'; track: SuperEventTrack; recipient?: string | null } |
  { kind: 'branch'; arms: { owner?: string | null; condition: ScriptCondition<CountryFlag>; effects: MusicEffect[] }[] };

interface TfrSuperEventSkin {
  frame: string | null;
  button: string | null;
}

interface TnoSuperEventSkin {
  frame: string | null;
  title: string | null;
  quote: string | null;
  button: string | null;
  nuclear_frame: string | null;
  nuclear_title: string | null;
  quote_font: string | null;
}

interface TnoBitmapGlyph {
  image: string;
  advance: number;
  width: number;
  height: number;
  left: number;
  bottom: number;
}

type TnoBitmapFont = Record<string, TnoBitmapGlyph>;

interface TnoSuperEventGlyphs {
  title: TnoBitmapFont;
  quote: TnoBitmapFont;
}

interface SuperEvent {
  id: string;
  title: string;
  quote: string | null;
  dismiss: string | null;
  image_path: string | null;
  audio: SuperEventTrack | null;
}

interface Country {
  flag_image?: string | null;
  major_order?: number | null;
  chapter_indices?: number[];
  tag: string;
  name: string;
  chapter_count: number;
  story_count: number;
}

interface ForeignEvent {
  recipient: string;
  target: string;
  delay: string;
}

interface FocusCompletion {
  bypass?: boolean;
  tag: string | null;
  id: string;
}

interface ExternalDevelopment {
  focus: Chapter;
  country: string;
  events: { item: TimelineItem; delay: number }[];
}

interface CountryFlag {
  flag: string;
  present: boolean;
}

interface Chapter {
  selection_music?: MusicEffect[];
  music?: MusicEffect[];
  bypass_condition?: ScriptCondition<CountryFlag>;
  background_availability?: CountryFlag[] | null;
  ai_weight?: number;
  never_manually_available?: boolean;
  mission_time_changes?: [string, number][];
  mission_activations?: string[];
  event_calls?: [string, string][];
  selection_calls?: [string, string][];
  selection_flags?: CountryFlag[];
  selection_missions?: string[];
  selection_completions?: FocusCompletion[];
  selection_routes?: ConditionalRoutes;
  selection_foreign_events?: ForeignEvent[];
  selection_broadcasts?: [string, string][];
  conditional_routes?: ConditionalRoutes;
  credits?: string[];
  broadcast_calls?: [string, string][];
  available_after_focuses?: string[];
  branch_flags?: CountryFlag[] | null;
  branch_condition?: ScriptCondition<CountryFlag>;
  availability_condition?: ScriptCondition<CountryFlag>;
  availability_flags?: CountryFlag[];
  flag_changes?: CountryFlag[];
  focus_tree_changes?: string[];
  focus_completions?: FocusCompletion[];
  foreign_events?: ForeignEvent[];
  super_event?: SuperEvent | null;
  id: string;
  title: string;
  description: string;
  icon: string;
  image_available: boolean;
  image_path: string | null;
  source: string;
  x?: number;
  y?: number;
  duration_days?: number;
  focus_tree_id?: string;
  country_tags?: string[];
  initial_tree?: boolean;
  branch_conditioned?: boolean;
  branch_available_at_start?: boolean;
  completed_at_start?: boolean;
  relative_position_id?: string;
  prerequisite_groups?: string[][];
  mutually_exclusive_ids?: string[];
}

interface StoryOption {
  effect_preview?: { lines: { text: string; context: string[] }[]; incomplete: boolean };
  music?: MusicEffect[];
  mission_time_changes?: [string, number][];
  mission_activations?: string[];
  trigger_checks?: CountryFlag[] | null;
  trigger_condition?: ScriptCondition<CountryFlag>;
  conditional_routes?: ConditionalRoutes;
  foreign_events?: ForeignEvent[];
  credits?: string[];
  event_calls?: [string, string][];
  flag_changes?: CountryFlag[];
  focus_tree_changes?: string[];
  route_effects?: string[];
  focus_completions?: FocusCompletion[];
  target_tags?: Record<string, string[]>;
  super_event?: SuperEvent | null;
  id: string;
  text: string;
  next_ids: string[];
}

interface TextVariant {
  text: string;
  flags: CountryFlag[];
  completed_focuses: string[];
}

interface ConditionalRoute {
  owner?: string;
  condition: ScriptCondition<CountryFlag>;
  trees: string[];
  completions: FocusCompletion[];
  calls?: [string, string][];
  foreign_events?: ForeignEvent[];
  flags?: CountryFlag[];
  missions?: string[];
  nested?: ConditionalRoutes;
}

interface WartimeHook {
  id: string;
  war: string;
  kind: string;
  source: string;
  arms: { condition: ScriptCondition<CountryFlag>; calls: ForeignEvent[]; flags: CountryFlag[] }[];
}

type ConditionalRoutes = ConditionalRoute[][];

interface MissionEffect {
  music?: MusicEffect[];
  calls: [string, string][];
  foreign_events: ForeignEvent[];
  conditional_routes?: ConditionalRoutes;
  activated_missions?: [string, string][];
  flags: CountryFlag[];
  trees: string[];
  completions: FocusCompletion[];
}

interface MissionResolution {
  id: string;
  label: string;
  effect: MissionEffect;
  flags?: CountryFlag[];
}

interface TimelineItem {
  music?: MusicEffect[];
  focus_completion_routes?: ConditionalRoutes;
  decision_end_music?: { days: number; effects: MusicEffect[]; flags: CountryFlag[]; conditional_routes: ConditionalRoutes };
  manual_trigger?: {
    owner: string; earliest: string | null; earliest_inclusive: boolean;
    latest: string | null; latest_inclusive: boolean; focuses: string[]; condition: ScriptCondition<CountryFlag>;
    prompt?: string | null;
  };
  focus_progress_trigger?: { focus: string; progress: number; inclusive: boolean };
  autonomous_trigger?: { owner: string; condition: ScriptCondition<CountryFlag>; mean_days: number | null };
  periodic_triggers?: {
    owner: string;
    recipient: string;
    condition: ScriptCondition<CountryFlag>;
    interval: number;
    delay: string;
  }[];
  automatic_timeout?: boolean;
  verified_startup?: ForeignEvent[];
  startup_trigger?: string | null;
  decision_guarded_calls?: { target: string; delay: string; checks: CountryFlag[]; deferred: boolean }[];
  hidden?: boolean;
  never_manually_available?: boolean;
  mission_time_changes?: [string, number][];
  mission_activations?: string[];
  mission?: { days: number; shared_resolution?: boolean; complete: MissionEffect; timeout: MissionEffect; cancel: MissionEffect } | null;
  immediate_calls?: [string, string][];
  trigger_checks?: CountryFlag[] | null;
  conditional_routes?: ConditionalRoutes;
  decision_checks?: CountryFlag[];
  decision_condition?: ScriptCondition<CountryFlag> | null;
  decision_focuses?: string[];
  decision_event_calls?: [string, string][];
  title_variants?: TextVariant[];
  description_variants?: TextVariant[];
  credits?: string[];
  broadcast?: boolean;
  broadcast_calls?: [string, string][];
  foreign_events?: ForeignEvent[];
  flag_changes?: CountryFlag[];
  focus_tree_changes?: string[];
  focus_completions?: FocusCompletion[];
  super_event?: SuperEvent | null;
  date_after?: boolean;
  date_before?: string | null;
  date_focuses?: string[];
  date_country?: string | null;
  id: string;
  kind: 'event' | 'news' | 'decision' | 'mission' | string;
  country_tag: string;
  title: string;
  description: string;
  date: string | null;
  timing: string;
  source: string;
  image_path: string | null;
  start_day: number | null;
  start_timing: string | null;
  next_ids: string[];
  linked_from: string[];
  options: StoryOption[];
  confirmation?: { choices: { label: string; route?: ConditionalRoute; event?: string; decision?: string;
    routes?: ConditionalRoute[]; flags?: CountryFlag[]; group?: string }[]; owner: string; projected: boolean; wartime?: boolean };
}

interface ScanResult {
  inline_assets?: Record<string, string>;
  text_colors?: Record<string, string>;
  opening_notes?: Record<string, { text: string; images: string[] }[]>;
  country_names?: Pick<Country, 'name' | 'tag' | 'major_order'>[];
  has_secondary_countries?: boolean;
  cache_warning?: string | null;
  wartime_hooks?: WartimeHook[];
  war_news_missions?: Record<string, string[]>;
  american_crisis_anchor?: string | null;
  great_asian_war_start?: ForeignEvent | null;
  great_asian_war_recipients?: ForeignEvent[];
  great_asian_war_news?: [string, string] | null;
  default_game_rules?: Record<string, string>;
  initial_country_flags?: Record<string, string[]>;
  start_date: string | null;
  countries: Country[];
  chapters: Chapter[];
  timeline: TimelineItem[];
  diagnostics: string[];
}

interface Saved {
  game_path: string;
  workshop_path: string;
  mod_path: string;
  tag: string;
  focus_id: string;
  last_kind: string;
  read_focus_ids: string[];
  excluded_focus_ids: string[];
  read_story_ids: string[];
  route_day: number;
  focus_completion_days: Record<string, number>;
  last_title: string;
  last_country: string;
  last_mod: string;
  language: Lang;
  music_transition: MusicTransition;
  theme: ThemePreference;
  theme_preference_set?: boolean;
  last_mod_theme?: ReaderTheme;
  music_history?: RememberedTrack[];
  route_history?: RouteCheckpoint[];
  option_due_days?: Record<string, number>;
  option_estimated_days?: Record<string, number>;
  country_flag_changes?: Record<string, boolean>;
  story_choices?: Record<string, string>;
  active_focus_tree?: string;
  decision_started?: Record<string, number>;
  mission_started?: Record<string, number>;
  mission_deadline_adjustments?: Record<string, number>;
  confirmations?: Record<string, { item: TimelineItem; day: number }>;
  wartime_starts?: Record<string, number>;
}

type RouteCheckpoint = {
  returnTo: 'focus-choice' | 'event';
  progress: Pick<
    Saved,
    | 'focus_id'
    | 'tag'
    | 'last_kind'
    | 'read_focus_ids'
    | 'excluded_focus_ids'
    | 'read_story_ids'
    | 'route_day'
    | 'focus_completion_days'
    | 'last_title'
    | 'last_country'
    | 'last_mod'
    | 'option_due_days'
    | 'option_estimated_days'
    | 'country_flag_changes'
    | 'story_choices'
    | 'active_focus_tree'
    | 'decision_started'
    | 'mission_started'
    | 'mission_deadline_adjustments'
    | 'confirmations'
    | 'wartime_starts'
  >;
};

const emptySaved: Saved = {
  game_path: '',
  workshop_path: '',
  mod_path: '',
  tag: '',
  focus_id: '',
  last_kind: 'focus',
  read_focus_ids: [],
  excluded_focus_ids: [],
  read_story_ids: [],
  route_day: 0,
  focus_completion_days: {},
  last_title: '',
  last_country: '',
  last_mod: '',
  language: 'english',
  music_transition: 'interrupt',
  theme: 'auto',
};

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [
    CommonModule,
    A11yModule,
    FormsModule,
    MatSidenavModule,
    MatToolbarModule,
    MatIconModule,
    MatButtonModule,
    MatCardModule,
    MatListModule,
    MatFormFieldModule,
    MatInputModule,
    MatSelectModule,
    MatProgressSpinnerModule,
    MatChipsModule,
    MatCheckboxModule,
    MatSlideToggleModule,
    MatBottomSheetModule,
  ],
  templateUrl: './app.html',
  styleUrls: ['./app.scss', './focus-picker.scss', './app-mobile.scss', './library.scss'],
})
export class App implements OnInit {
  private readonly breakpoints = inject(BreakpointObserver);
  readonly compactNavigation = toSignal(this.breakpoints.observe('(max-width: 760px)').pipe(map((state) => state.matches)));
  readonly touchEffects = toSignal(this.breakpoints.observe('(hover: none)').pipe(map((state) => state.matches)));
  readonly android = /Android/i.test(navigator.userAgent);
  private readonly bottomSheet = inject(MatBottomSheet);
  private optionEffectsSheet?: MatBottomSheetRef<unknown>;
  importingArchive = false;
  showDownloadForm = false;
  downloadUrl = '';
  downloadProgress: DownloadProgress | null = null;
  searchQuery = '';
  searchResults: SearchResult[] = [];
  searchCount = 0;
  private searchDocuments: SearchDocument[] | null = null;
  private searchLimit = 40;
  searchPreview = false;
  private searchMatch?: SearchField;
  effectTooltip: { text: string; x: number; y: number; left: boolean; above: boolean } | null = null;
  private inlineAssets: Record<string, string> = {};
  private textColors: Record<string, string> = {};
  private readonly inlineImages = new Map<string, string>();
  private readonly gameTextCache = new Map<string, { text: string; color?: string; icon?: string; flag?: boolean }[]>();
  private readonly changeDetector = inject(ChangeDetectorRef);
  page: Page = 'home';
  saved: Saved = { ...emptySaved };
  private startDate: string | null = null;
  mods: ModInfo[] = [];
  countries: Country[] = [];
  private openingNotes: NonNullable<ScanResult['opening_notes']> = {};
  countryPreview: Country[] = [];
  hasSecondaryCountries = false;
  secondaryIndexPending = false;
  cacheWarning = '';
  private allIndexedCountries: Country[] | null = null;
  private countryNames = new Map<string, string>();
  pendingCountryTag: string | null = null;
  private loadedScanKey: string | null = null;
  private activeScan: { key: string; result: Promise<boolean> } | null = null;
  private scanVersion = 0;
  private sortedCountrySource: Country[] | null = null;
  private sortedCountries: Country[] = [];
  chapters: Chapter[] = [];
  timeline: TimelineItem[] = [];
  diagnostics: string[] = [];
  selectedMod: ModInfo | null = null;
  selectedCountry: Country | null = null;
  current: Chapter | TimelineItem | null = null;
  image = '';
  selectedOption: StoryOption | null = null;
  branchMessage = '';
  loading = false;
  query = '';
  countryQuery = '';
  showAllCountries = false;
  error = '';
  superEvent: SuperEvent | null = null;
  superEventImage = '';
  superEventImageUnavailable = false;
  superEventSkin: TfrSuperEventSkin | null = null;
  tnoSuperEventSkin: TnoSuperEventSkin | null = null;
  tnoSuperEventGlyphs: TnoSuperEventGlyphs | null = null;
  tnoSuperEventTextReady = false;
  bgmNotice = '';
  audioStatus = '';
  audioNeedsGesture = false;
  private superEventContinuation: (() => void) | null = null;
  private superEventTrigger: HTMLElement | null = null;
  private readonly tfrSuperEventSkins = new Map<string, Promise<TfrSuperEventSkin | null>>();
  private readonly tnoSuperEventSkins = new Map<string, Promise<TnoSuperEventSkin | null>>();
  private readonly tnoGlyphRequests = new Map<string, Promise<TnoSuperEventGlyphs | null>>();
  private readonly audioQueue: SuperEventTrack[] = [];
  private playingAudio: HTMLAudioElement | null = null;
  private playingTrack: SuperEventTrack | null = null;
  private bgmTimer: ReturnType<typeof setTimeout> | undefined;

  private openSuperEvent(scene: SuperEvent | null | undefined, continuation?: () => void) {
    if (!scene) {
      continuation?.();
      return;
    }
    this.superEventTrigger =
      document.activeElement instanceof HTMLElement ? document.activeElement : null;
    this.superEvent = scene;
    this.superEventImage = '';
    this.superEventImageUnavailable = false;
    this.superEventSkin = null;
    this.tnoSuperEventSkin = null;
    this.tnoSuperEventGlyphs = null;
    this.tnoSuperEventTextReady = false;
    this.superEventContinuation = continuation ?? null;
    this.changeDetector.markForCheck();
    this.focusSuperEvent(scene);
    if (this.theme === 'tfr') void this.loadTfrSuperEventSkin(scene);
    if (this.theme === 'tno') void this.loadTnoSuperEventSkin(scene);
    if (scene.image_path && this.selectedMod) {
      void invoke<string | null>('load_super_event_image', {
        modPath: this.selectedMod.path,
        imagePath: scene.image_path,
      })
        .then((image) => {
          if (this.superEvent === scene) {
            this.superEventImage = image || '';
            this.superEventImageUnavailable = !image;
            this.changeDetector.markForCheck();
          }
        })
        .catch(() => {
          if (this.superEvent === scene) {
            this.superEventImageUnavailable = true;
            this.changeDetector.markForCheck();
          }
        });
    }
    if (scene.audio) this.enqueueSuperEventAudio(scene.audio);
  }

  onSuperEventImageError() {
    this.superEventImage = '';
    this.superEventImageUnavailable = true;
  }

  private async loadTfrSuperEventSkin(scene: SuperEvent) {
    const path = this.mods.find((mod) => mod.workshop_id === '3350890356')?.path ?? this.selectedMod?.path;
    if (!path) return;
    let request = this.tfrSuperEventSkins.get(path);
    if (!request) {
      request = invoke<TfrSuperEventSkin | null>('load_tfr_super_event_skin', { modPath: path }).catch(() => {
        this.tfrSuperEventSkins.delete(path);
        return null;
      });
      this.tfrSuperEventSkins.set(path, request);
    }
    const skin = await request;
    if (this.superEvent === scene) {
      this.superEventSkin = skin;
      this.changeDetector.markForCheck();
    }
  }

  get tnoNuclearSuperEvent() {
    return this.superEvent?.id === 'SE_NUCLEAR_WAR';
  }

  private focusSuperEvent(scene: SuperEvent) {
    setTimeout(() => {
      if (this.superEvent !== scene) return;
      const focusId = this.theme === 'tno' && !this.tnoSuperEventTextReady
        ? 'super-event-dialog' : 'super-event-continue';
      document.getElementById(focusId)?.focus();
    }, 0);
  }

  private async loadTnoSuperEventSkin(scene: SuperEvent) {
    const selected = this.selectedMod;
    const path = selected && this.themeForMod(selected.workshop_id, selected.name) === 'tno'
      ? selected.path
      : this.mods.find((mod) => mod.workshop_id === '2438003901')?.path ?? selected?.path;
    const [skin, glyphs] = path ? await Promise.all([
      this.requestTnoSuperEventSkin(path),
      this.requestTnoSuperEventGlyphs(path, scene),
    ]) : [null, null];
    if (this.superEvent !== scene) return;
    this.tnoSuperEventSkin = skin;
    this.tnoSuperEventGlyphs = glyphs;
    this.tnoSuperEventTextReady = true;
    this.changeDetector.markForCheck();
    this.focusSuperEvent(scene);
  }

  private requestTnoSuperEventGlyphs(path: string, scene: SuperEvent) {
    const title = this.plainGameText(scene.title);
    const quoteAndButton = this.plainGameText(`${scene.quote ?? ''}\n${scene.dismiss || this.t('继续阅读')}`);
    if (this.readerLanguage === 'ja' || !/[\u2e80-\u9fff\uf900-\ufaff\uff00-\uffef]/u.test(title + quoteAndButton)) {
      return null;
    }
    const key = JSON.stringify([path, title, quoteAndButton]);
    let request = this.tnoGlyphRequests.get(key);
    if (request) return request;
    if (this.tnoGlyphRequests.size >= 16) this.tnoGlyphRequests.clear();
    request = invoke<TnoSuperEventGlyphs>('load_tno_super_event_glyphs', { modPath: path, title, quoteAndButton })
      .then(async (glyphs) => {
        await Promise.all([glyphs.title, glyphs.quote].flatMap((font) =>
          Object.entries(font).map(async ([character, glyph]) => {
            const image = new Image();
            if (typeof image.decode !== 'function') return;
            image.src = glyph.image;
            await image.decode().catch(() => { delete font[character]; });
          }),
        ));
        return glyphs;
      }).catch(() => {
        this.tnoGlyphRequests.delete(key);
        return null;
      });
    this.tnoGlyphRequests.set(key, request);
    return request;
  }

  private requestTnoSuperEventSkin(path: string) {
    let request = this.tnoSuperEventSkins.get(path);
    if (!request) {
      request = invoke<TnoSuperEventSkin | null>('load_tno_super_event_skin', { modPath: path })
        .then(async (skin) => {
          if (skin?.quote_font && typeof FontFace !== 'undefined') {
            const font = new FontFace('TNO Super Event Quote', `url(${JSON.stringify(convertFileSrc(skin.quote_font))})`);
            await font.load().then((loaded) => document.fonts.add(loaded)).catch(() => {});
          }
          return skin;
        }).catch(() => {
          this.tnoSuperEventSkins.delete(path);
          return null;
        });
      this.tnoSuperEventSkins.set(path, request);
    }
    return request;
  }

  closeSuperEvent() {
    const continuation = this.superEventContinuation;
    const trigger = this.superEventTrigger;
    this.superEvent = null;
    this.superEventContinuation = null;
    this.superEventTrigger = null;
    continuation?.();
    this.changeDetector.markForCheck();
    setTimeout(() => {
      if (!this.superEvent) {
        if (trigger?.isConnected) trigger.focus();
        else document.getElementById('reader-heading')?.focus();
      }
    }, 0);
  }

  private enqueueSuperEventAudio(track: SuperEventTrack) {
    if (this.playingTrack?.path === track.path) return;
    if (this.saved.music_transition === 'interrupt') {
      this.audioQueue.length = 0;
      this.stopPlayingAudio();
    } else if (this.audioQueue.some((item) => item.path === track.path)) {
      return;
    }
    this.audioQueue.push(track);
    if (!this.playingAudio) this.playNextSuperEventAudio();
  }

  setMusicTransition(mode: MusicTransition) {
    this.saved.music_transition = mode;
    if (mode === 'interrupt') {
      const latest = this.audioQueue.pop();
      this.audioQueue.length = 0;
      if (latest) this.enqueueSuperEventAudio(latest);
    }
    this.persist();
  }

  get theme(): ReaderTheme {
    return this.saved.theme === 'auto'
      ? (this.saved.last_mod_theme ?? this.themeForMod('', this.saved.last_mod))
      : this.saved.theme;
  }

  private themeForMod(id: string, name: string): ReaderTheme {
    if (['3350890356', '3609728809'].includes(id) || /the fire rises/i.test(name)) return 'tfr';
    if (['2438003901', '3256452254'].includes(id) || /the new order|道阻且长|道阻且長/i.test(name)) return 'tno';
    return 'material';
  }

  private applyTheme() {
    document.documentElement.dataset['readerTheme'] = this.theme;
    void this.loadTfrMenuArt();
  }

  setTheme(theme: ThemePreference) {
    this.saved.theme = theme;
    this.saved.theme_preference_set = true;
    this.applyTheme();
    this.persist();
  }

  tfrMenuArt = '';
  private tfrMenuArtPath = '';

  private async loadTfrMenuArt() {
    if (this.theme !== 'tfr') return;
    const path = this.mods.find((mod) => mod.workshop_id === '3350890356')?.path ?? this.selectedMod?.path;
    if (!path || path === this.tfrMenuArtPath) return;
    this.tfrMenuArtPath = path;
    this.tfrMenuArt = '';
    try {
      const image = await invoke<string | null>('load_tfr_menu_art', { modPath: path });
      if (this.tfrMenuArtPath === path) this.tfrMenuArt = image ?? '';
    } catch {
      if (this.tfrMenuArtPath === path) this.tfrMenuArt = '';
    } finally {
      this.changeDetector.markForCheck();
    }
  }

  private stopPlayingAudio() {
    const audio = this.playingAudio;
    if (!audio) return;
    this.playingAudio = null;
    this.playingTrack = null;
    clearTimeout(this.bgmTimer);
    this.bgmNotice = '';
    this.audioNeedsGesture = false;
    audio.pause();
    audio.removeAttribute('src');
    audio.load();
  }

  get musicHistory() {
    return this.saved.music_history ?? [];
  }

  isPlayingTrack(track: SuperEventTrack) {
    return this.playingTrack?.path === track.path;
  }

  replayTrack(track: RememberedTrack) {
    this.audioQueue.length = 0;
    const alreadyPlaying = this.isPlayingTrack(track) && !this.audioNeedsGesture;
    this.stopPlayingAudio();
    this.audioStatus = '';
    if (alreadyPlaying) return;
    this.audioQueue.push(track);
    this.playNextSuperEventAudio();
  }

  private playScriptedMusic(effects: MusicEffect[] = [], owner = this.saved.tag, day = this.routeDay) {
    for (const effect of effects) {
      if (effect.kind === 'play') {
        if ((effect.recipient ?? owner) === this.saved.tag) this.enqueueSuperEventAudio(effect.track);
        continue;
      }
      for (const arm of effect.arms) {
        const scope = arm.owner ?? owner;
        const result = this.evaluateForCountry(arm.condition, scope, day);
        // An unresolved earlier test could select different music.
        if (result === null) break;
        if (result) {
          this.playScriptedMusic(arm.effects, scope, day);
          break;
        }
      }
    }
  }

  private playNextSuperEventAudio() {
    const track = this.audioQueue.shift();
    if (!track) return;
    const audio = new Audio(convertFileSrc(track.path));
    this.playingAudio = audio;
    this.playingTrack = track;
    this.audioStatus = '';
    this.audioNeedsGesture = false;
    const finish = (failed: boolean) => {
      if (this.playingAudio !== audio) return;
      this.playingAudio = null;
      this.playingTrack = null;
      this.audioNeedsGesture = false;
      this.audioStatus = failed ? '无法播放这段本地音频。' : '';
      this.changeDetector.markForCheck();
      this.playNextSuperEventAudio();
    };
    audio.addEventListener('ended', () => finish(false), { once: true });
    audio.addEventListener('error', () => finish(true), { once: true });
    void this.startSuperEventAudio(audio, track);
  }

  retrySuperEventAudio() {
    if (this.playingAudio && this.playingTrack) {
      void this.startSuperEventAudio(this.playingAudio, this.playingTrack);
    }
  }

  private async startSuperEventAudio(audio: HTMLAudioElement, track: SuperEventTrack) {
    try {
      await audio.play();
      if (this.playingAudio !== audio) return;
      this.audioNeedsGesture = false;
      this.audioStatus = '';
      const played = this.musicHistory.find((item) => item.path === track.path);
      const mod = played?.mod || this.selectedMod?.name || this.saved.last_mod;
      this.saved.music_history = [{ title: track.title, path: track.path, mod },
        ...this.musicHistory.filter((item) => item.path !== track.path)].slice(0, 100);
      this.persist();
      clearTimeout(this.bgmTimer);
      this.bgmNotice = track.title;
      this.bgmTimer = setTimeout(() => {
        this.bgmNotice = '';
        this.changeDetector.markForCheck();
      }, 4400);
    } catch (error) {
      if (this.playingAudio !== audio) return;
      if (error instanceof DOMException && error.name === 'NotAllowedError') {
        this.audioNeedsGesture = true;
      } else {
        this.playingAudio = null;
        this.playingTrack = null;
        this.audioStatus = '无法播放这段本地音频。';
        this.playNextSuperEventAudio();
      }
    }
    this.changeDetector.markForCheck();
  }

  storyPreview = false;
  private americanCrisisAnchor: string | null = null;
  private greatAsianWarNews: [string, string] | null = null;
  private greatAsianWarStart: ForeignEvent | null = null;
  private greatAsianWarRecipients: ForeignEvent[] = [];
  private defaultGameRules: Record<string, string> = {};
  private initialCountryFlags: Record<string, string[]> = {};
  private focusById = new Map<string, Chapter>();
  private timelineById = new Map<string, TimelineItem[]>();
  private timelineByKey = new Map<string, TimelineItem>();
  private countryTimelineCache: TimelineItem[] = [];
  private externalByTag = new Map<string, ExternalDevelopment[]>();
  private foreignStoryDays = new Map<string, number>();
  private worldNewsDays = new Map<string, Map<string, number>>();
  private projectedStoryDays = new Map<string, number>();
  private europeanWarDay: number | null = null;
  private projectedNewsDays = new Map<string, Map<string, number>>();
  private readonly checkedDecisions = new Set<string>();
  private countryTimelineCacheTag: string | null = null;
  private scheduleCache = new Map<string, number | null>();
  private scheduleCacheFocusSignature = '';

  readonly langOptions: { value: Lang; label: string }[] = [
    { value: 'english', label: 'English' },
    { value: 'simp_chinese', label: '简体中文' },
    { value: 'traditional_chinese', label: '正體中文' },
    { value: 'japanese', label: '日本語' },
  ];

  t(text: string, ...values: (number | string)[]): string {
    const translations = UI_TEXT[text];
    const copy = this.saved.language === 'simp_chinese' || !translations ? text :
      translations[this.saved.language === 'japanese' ? 2 : this.saved.language === 'traditional_chinese' ? 1 : 0];
    return copy.replace(/\{(\d+)\}/g, (_, index) => String(values[Number(index)] ?? ''));
  }

  setLanguage(language: Lang) {
    this.saved.language = language;
    document.documentElement.lang = this.readerLanguage;
    this.persist();
    if (this.selectedMod) void this.scan();
  }

  get visibleMods() {
    const query = this.query.trim().toLocaleLowerCase();
    return this.mods.filter((mod) => mod.name.toLocaleLowerCase().includes(query));
  }

  get countryIndexPending() {
    return this.activeScan !== null;
  }

  get offersSecondaryCountries() {
    return this.hasSecondaryCountries || this.countryPreview.some((country) => country.major_order != null);
  }

  get listedCountries() {
    if (this.showAllCountries && this.allIndexedCountries) return this.allIndexedCountries;
    return this.countries.length ? this.countries : this.countryPreview;
  }

  get visibleCountries() {
    const query = this.countryQuery.trim().toLocaleLowerCase();
    const countries = this.listedCountries;
    if (countries !== this.sortedCountrySource) {
      this.sortedCountrySource = countries;
      this.sortedCountries = [...countries].sort(
        (a, b) =>
          (a.major_order ?? Number.MAX_SAFE_INTEGER) - (b.major_order ?? Number.MAX_SAFE_INTEGER) ||
          b.chapter_count + b.story_count - (a.chapter_count + a.story_count) ||
          a.name.localeCompare(b.name),
      );
    }
    return this.sortedCountries.filter(
      (country) =>
        (country.major_order != null || this.countryIndexPending || this.showAllCountries || country.chapter_count > 0 || country.story_count > 0) &&
        (!query || country.tag.toLocaleLowerCase().includes(query) || country.name.toLocaleLowerCase().includes(query)),
    );
  }

  get countryChapters() {
    const country = (this.allIndexedCountries ?? this.countries).find(({ tag }) => tag === this.saved.tag);
    if (country?.chapter_indices) return country.chapter_indices.map((index) => this.chapters[index]);
    return this.chapters.filter((chapter) => this.focusBelongsTo(chapter, this.saved.tag));
  }

  private focusBelongsTo(chapter: Chapter, tag: string) {
    return chapter.country_tags?.length ? chapter.country_tags.includes(tag) : chapter.id.startsWith(`${tag}_`);
  }

  private focusVariants = new Map<string, Chapter[]>();

  private localFocus(id: string) {
    const candidates = (this.focusVariants.get(id) ?? []).filter((chapter) => this.focusBelongsTo(chapter, this.saved.tag));
    return candidates.find((chapter) => this.saved.active_focus_tree
      ? chapter.focus_tree_id === this.saved.active_focus_tree : chapter.initial_tree) ?? candidates[0];
  }

  get activeFocusTreeIds() {
    if (this.saved.active_focus_tree) return new Set([this.saved.active_focus_tree]);
    return new Set(
      this.countryChapters
        .filter((chapter) => chapter.initial_tree)
        .map((chapter) => chapter.focus_tree_id),
    );
  }

  get routeFocuses() {
    const initialTreeIds = this.activeFocusTreeIds;
    return initialTreeIds.size
      ? this.countryChapters.filter((chapter) => initialTreeIds.has(chapter.focus_tree_id))
      : [];
  }

  get openingWindowDays() {
    const roots = this.countryChapters.filter(
      (focus) => focus.initial_tree && !(focus.prerequisite_groups || []).length,
    );
    return roots.length ? Math.min(...roots.map((focus) => focus.duration_days || 70)) : 70;
  }

  get routeDay() {
    return Math.max(this.saved.route_day || 0, this.openingWindowDays);
  }

  get orderedFocuses() {
    return this.sortFocuses(this.routeFocuses);
  }

  get focusCandidates() {
    if (this.dueStories.length) return [];
    const available = this.availableFocuses;
    const wartime = this.activeWarMission && available.filter(({ id }) => this.warFocusIds.has(id));
    return this.firstFocusRow(wartime?.length ? wartime : available);
  }

  private get availableFocuses() {
    const completed = new Set(this.saved.read_focus_ids || []);
    const excluded = new Set(this.saved.excluded_focus_ids || []);
    return this.routeFocuses.filter((chapter) => {
      if (chapter.never_manually_available || completed.has(chapter.id) || excluded.has(chapter.id)) return false;
      if (chapter.mutually_exclusive_ids?.some((id) => completed.has(id))) return false;
      if (!this.focusFlagsAllow(chapter)) return false;
      if (!(chapter.available_after_focuses ?? []).every((id) => completed.has(id))) return false;
      const groups = chapter.prerequisite_groups || [];
      return groups.every((group) =>
        group.some((parent) => completed.has(parent) || !!this.localFocus(parent)?.completed_at_start),
      );
    });
  }

  private firstFocusRow(available: Chapter[]) {
    if (!available.length) return [];
    const firstAvailableRow = Math.min(
      ...available.map((focus) => this.absoluteFocusPosition(focus).y),
    );
    const choices = available.filter((focus) => this.absoluteFocusPosition(focus).y === firstAvailableRow);
    const included = new Set(choices.map(({ id }) => id));
    for (let index = 0; index < choices.length; index++) {
      const focus = choices[index];
      for (const other of available) {
        if (included.has(other.id) || !this.focusesConflict(focus, other)) continue;
        included.add(other.id);
        choices.push(other);
      }
    }
    return this.sortFocuses(choices);
  }

  private focusesConflict(a: Chapter, b: Chapter) {
    return a.mutually_exclusive_ids?.includes(b.id) || b.mutually_exclusive_ids?.includes(a.id);
  }

  private get activeWarMission() {
    const tag = this.saved.tag;
    if (tag !== 'PRC' && tag !== 'JAP') return undefined;
    const id = `${tag}_GAW_war_exhaustion`;
    return this.saved.mission_started?.[id] !== undefined &&
      !this.saved.read_story_ids.includes(`mission:${id}`) ? this.timelineByKey.get(`mission:${id}`) : undefined;
  }

  private get warFocusIds() {
    const root = this.saved.tag === 'JAP' ? 'JAP_second_war_in_greater_east_asia'
      : this.saved.read_focus_ids.includes('PRC_new_asian_liberation_war')
        ? 'PRC_new_asian_liberation_war' : 'PRC_chinas_greatest_hour';
    const ids = new Set([root]);
    const route = this.routeFocuses;
    for (let added = true; added;) {
      added = false;
      for (const focus of route) {
        if (ids.has(focus.id) || !focus.prerequisite_groups?.some(
          (group) => group.length && group.every((id) => ids.has(id)),
        )) continue;
        ids.add(focus.id);
        added = true;
      }
    }
    return ids;
  }

  focusChoiceRows: { focus: Chapter; image: string; exclusiveWith: string[] }[][] = [];

  get immersiveReading() {
    return this.page === 'reader' || this.page === 'focus-choice';
  }

  get blockedFocuses() {
    const completed = new Set(this.saved.read_focus_ids || []);
    const known = new Set(this.routeFocuses.map((chapter) => chapter.id));
    return this.sortFocuses(
      this.routeFocuses.filter(
        (chapter) =>
          !completed.has(chapter.id) &&
          (chapter.prerequisite_groups || []).some((group) =>
            group.every((parent) => known.has(parent) && !completed.has(parent)),
          ),
      ),
    );
  }

  private focusFlagsAllow(chapter: Chapter) {
    const matches = (check: CountryFlag) => this.flagMatches(check);
    const branchOpen = chapter.branch_condition
      ? evaluateCondition(chapter.branch_condition, matches) === true
      : chapter.branch_flags
      ? chapter.branch_flags.every(matches)
      : !chapter.branch_conditioned || chapter.branch_available_at_start || chapter.completed_at_start;
    if (!branchOpen) return false;
    const availability = chapter.availability_condition;
    return availability
      ? evaluateCondition(availability, (check) => {
          if (check.flag.startsWith('global:') &&
              !chapter.availability_flags?.some(({ flag }) => flag === check.flag) &&
              this.saved.country_flag_changes?.[check.flag] === undefined) return null;
          return matches(check);
        }) !== false
      : (chapter.availability_flags ?? []).every(matches);
  }

  private flagMatches({ flag, present }: CountryFlag, day = this.routeDay) {
    if (flag.startsWith('reader:')) {
      const value = this.saved.country_flag_changes?.[flag];
      return value === undefined ? null : value === present;
    }
    if (flag.startsWith('focus:')) {
      const id = flag.slice(6);
      const separator = id.indexOf(':');
      if (separator !== -1) {
        const tag = id.slice(0, separator);
        const focusId = id.slice(separator + 1);
        if (tag !== this.saved.tag) {
          const projected = this.projectedFocusDays.get(`${tag}:${focusId}`);
          return projected === undefined || projected > day ? null : present;
        }
        const completion = this.saved.focus_completion_days[focusId];
        return (completion !== undefined ? completion <= day :
          this.saved.read_focus_ids.includes(focusId) || !!this.localFocus(focusId)?.completed_at_start) === present;
      }
      return (this.saved.read_focus_ids.includes(id) || !!this.focusById.get(id)?.completed_at_start) === present;
    }
    if (flag.startsWith('tag:')) return (this.saved.tag === flag.slice(4)) === present;
    if (flag === 'is_ai') return !present;
    if (flag.startsWith('rule:')) {
      const [rule, option] = flag.slice(5).split('=');
      const selected = this.defaultGameRules[rule];
      return selected === undefined ? null : (selected === option) === present;
    }
    const initial = this.initialCountryFlags[this.saved.tag] ?? [];
    return (this.saved.country_flag_changes?.[flag] ?? initial.includes(flag)) === present;
  }

  private checksPass(checks: CountryFlag[] = []) {
    return checks.every((check) => this.flagMatches(check) === true);
  }

  private evaluateForCountry(condition: ScriptCondition<CountryFlag>, owner: string, day: number, missingGlobalUnknown = false) {
    return evaluateCondition(condition, (check) => {
      if (missingGlobalUnknown && check.flag.startsWith('global:') &&
          this.saved.country_flag_changes?.[check.flag] === undefined) return null;
      if (owner === this.saved.tag || check.flag.startsWith('global:') || check.flag.startsWith('rule:') ||
          /^focus:[A-Z0-9]{3}:/.test(check.flag)) return this.flagMatches(check, day);
      if (check.flag.startsWith('tag:')) return (owner === check.flag.slice(4)) === check.present;
      if (check.flag === 'is_ai') return check.present;
      return null;
    });
  }

  private applyConditionalRoutes(routes: ConditionalRoutes = [], day = this.routeDay, owner = this.saved.tag,
    projected = false, source?: TimelineItem | Chapter, path = '') {
    this.queueEnvironmentalOutcome(routes, day, owner, projected, source);
    for (const [chainIndex, chain] of routes.entries()) {
      const key = `${path}.${chainIndex}`;
      const possible: ConditionalRoute[] = [];
      let canSkip = true;
      for (const [index, route] of chain.entries()) {
        const matches = this.evaluateForCountry(route.condition, route.owner ?? owner, day,
          !!(source && 'kind' in source && source.id === 'second_nato_war_mission'));
        if (matches === false) continue;
        if (matches === true && !possible.length) {
          this.applyConditionalArm(route, day, owner, projected, source, `${key}.${index}`);
          break;
        }
        possible.push(route);
        if (matches === true) { canSkip = false; break; }
      }
      if (!source || !possible.length) continue;
      if (possible.some((route) => this.environmentThreshold(route))) continue;
      const choices = possible.map((route) => ({ route, label: this.conditionalResultLabel(route, owner) }));
      if (!choices.some(({ label }) => label)) continue;
      const labels = choices.map(({ route, label }, index) => ({ route, label: label || `其他结果 ${index + 1}` }));
      this.queueConfirmation(source, key, day, owner, projected,
        canSkip ? [...labels, { label: '条件未达成，继续阅读' }] : labels);
    }
  }

  private environmentThreshold(route: ConditionalRoute): string | undefined {
    const find = (condition: ScriptCondition<CountryFlag>): string | undefined => {
      if (condition.op === 'check' && condition.args.flag.startsWith('reader:PRC_environment_')) return condition.args.flag;
      if (condition.op === 'all') return condition.args.map(find).find(Boolean);
      return undefined;
    };
    return find(route.condition);
  }

  private queueEnvironmentalOutcome(routes: ConditionalRoutes, day: number, owner: string, projected: boolean, source?: TimelineItem | Chapter) {
    if (!source || this.saved.tag !== 'PRC' || owner !== 'PRC') return;
    const thresholds = routes.flat().filter((route) => this.environmentThreshold(route));
    if (!thresholds.length || this.saved.country_flag_changes?.['reader:PRC_environment_30'] !== undefined) return;
    const flags = (moderate: boolean, severe: boolean) => [
      { flag: 'reader:PRC_environment_30', present: moderate },
      { flag: 'reader:PRC_environment_20', present: severe },
    ];
    this.queueConfirmation(source, 'environment', day, owner, projected, [
      { label: '环保进展保持在 30% 以上', flags: flags(false, false) },
      { label: '环保进展降至 20% 至 30% 之间', flags: flags(true, false),
        routes: thresholds.filter((route) => this.environmentThreshold(route) === 'reader:PRC_environment_30') },
      { label: '环保进展降至 20% 或以下', flags: flags(true, true), routes: thresholds },
    ], { id: 'PRC_environment_outcome', title: '环保改革的结果', description: '环保进展降到了什么程度？' });
  }

  private applyConditionalArm(route: ConditionalRoute, day: number, owner: string, projected: boolean,
    source?: TimelineItem | Chapter, path = '') {
    owner = route.owner ?? owner;
    const foreign = owner !== this.saved.tag;
    if (foreign) {
      this.applyFlagChanges(route.flags?.filter(({ flag }) => flag.startsWith('global:')));
      this.scheduleScopedCalls(route.calls?.map(([target, delay]) => ({ recipient: owner, target, delay })), day, projected);
    } else {
      this.applyFlagChanges(route.flags);
      this.applyTreeChanges(route.trees);
      this.applyFocusCompletions(route.completions, day, projected);
      this.activateMissions(route.missions, day);
      this.scheduleScriptCalls(route.calls, day, projected);
    }
    this.scheduleScopedCalls(route.foreign_events, day, projected);
    this.applyConditionalRoutes(route.nested, day, owner, projected, source, path);
  }

  private conditionalResultLabel(route: ConditionalRoute, owner: string): string {
    owner = route.owner ?? owner;
    const ids = [...(route.calls ?? []).map(([target]) => ({ target, recipient: owner })), ...(route.foreign_events ?? [])];
    const titles = ids.flatMap(({ target, recipient }) => (this.timelineById.get(target.toLowerCase()) ?? [])
      .filter((item) => (recipient === this.saved.tag || item.broadcast) && !item.hidden && ['event', 'news'].includes(item.kind))
      .map((item) => item.title.startsWith('[Missing translation:') ? item.id : item.title));
    if (owner === this.saved.tag) {
      for (const completion of route.completions) {
        if (completion.tag && completion.tag !== owner) continue;
        const focus = this.localFocus(completion.id);
        if (focus) titles.push(focus.title);
      }
      for (const mission of route.missions ?? []) {
        const item = this.timelineByKey.get(`mission:${mission}`);
        if (item) titles.push(item.title);
      }
      for (const { flag, present } of route.flags ?? []) {
        if (!present || !this.focusConditionFlags.has(flag)) continue;
        const unlock = this.routeFocuses.find((focus) => !this.saved.read_focus_ids.includes(focus.id) &&
          [focus.branch_condition, focus.availability_condition].some((condition) => condition &&
            evaluateCondition(condition, (check) => check.flag === flag ? check.present : this.flagMatches(check)) === true &&
            evaluateCondition(condition, (check) => check.flag === flag ? !check.present : this.flagMatches(check)) !== true));
        if (unlock) titles.push(unlock.title);
      }
    }
    for (const chain of route.nested ?? []) for (const child of chain) {
      const label = this.conditionalResultLabel(child, owner);
      if (label) titles.push(label);
    }
    return [...new Set(titles)].slice(0, 3).join('、');
  }

  private queueConfirmation(source: TimelineItem | Chapter, path: string, day: number, owner: string, projected: boolean,
    choices: NonNullable<TimelineItem['confirmation']>['choices'], context?: { id?: string; title: string; description: string; wartime?: boolean }) {
    const sourceKey = 'kind' in source ? this.storyKey(source) : `focus:${source.id}`;
    const id = context?.id ?? `${sourceKey}:${path}:${day}`;
    const key = `confirmation:${id}`;
    const saved = this.saved.confirmations ??= {};
    if (this.saved.read_story_ids.includes(key) || saved[key]) return;
    saved[key] = { day, item: {
      id, kind: 'confirmation', country_tag: this.saved.tag, title: context?.title ?? source.title,
      description: context?.description ?? ('kind' in source && path === 'trigger' ? source.manual_trigger?.prompt : undefined) ??
        '请选择此时发生的结果。你的选择会决定接下来的剧情。',
      date: null, timing: '根据此处脚本的触发条件选择', source: source.source, image_path: null,
      start_day: day, start_timing: null, linked_from: [], next_ids: [], options: [],
      confirmation: { choices, owner, projected, wartime: context?.wartime },
    } };
  }

  private scheduleScriptCalls(calls: (readonly [string, string])[] = [], day: number, projected = false) {
    for (const [id, rawDelay] of calls) {
      for (const item of this.timelineById.get(id.toLowerCase()) ?? []) {
        this.scheduleEvent(item, rawDelay, day, projected);
      }
    }
  }

  private scheduleEvent(item: TimelineItem, rawDelay: string, day: number, projected = false) {
    if (!['event', 'news'].includes(item.kind) ||
        (item.startup_trigger && item.startup_trigger !== this.saved.tag)) return;
    const delay = this.parseStoryDelay(rawDelay);
    if (delay === null) return;
    const key = this.storyKey(item);
    if (this.saved.read_story_ids.includes(key)) return;
    const due = day + delay;
    const pending = this.saved.option_due_days ??= {};
    const estimated = projected || this.parseExactDelay(rawDelay) === null;
    if (due > (pending[key] ?? Infinity) || (due === pending[key] && estimated)) return;
    pending[key] = due;
    const estimates = this.saved.option_estimated_days ??= {};
    if (estimated) estimates[key] = due;
    else delete estimates[key];
  }

  private scheduleWorldConsequences(effect: MissionEffect, owner: string, day: number) {
    const queue = effect.calls.map(([target, delay]) => ({ target, delay, day }));
    const visited = new Set<string>();
    for (let i = 0; i < queue.length; i++) {
      const step = queue[i];
      const delay = this.parseStoryDelay(step.delay);
      if (delay === null) continue;
      for (const item of this.timelineById.get(step.target.toLowerCase()) ?? []) {
        const key = this.storyKey(item);
        if (visited.has(key) || !['event', 'news'].includes(item.kind) ||
            (item.startup_trigger && item.startup_trigger !== owner)) continue;
        visited.add(key);
        const due = step.day + delay;
        if (item.broadcast) this.scheduleEvent(item, step.delay, step.day, true);
        const calls = this.automaticEventCalls(item);
        this.scheduleScopedCalls(calls.foreign, due, true);
        for (const [target, delay] of calls.local) {
          if (!item.broadcast || this.timelineById.get(target.toLowerCase())?.some((target) => target.broadcast)) {
            queue.push({ target, delay, day: due });
          }
        }
      }
    }
  }

  private applyFlagChanges(changes: CountryFlag[] = []) {
    if (!changes.length) return;
    this.saved.country_flag_changes ??= {};
    for (const { flag, present } of changes) this.saved.country_flag_changes[flag] = present;
  }

  private wartimeHooks: WartimeHook[] = [];

  private japaneseDefeatCredits(item: TimelineItem) {
    if (this.saved.tag !== 'JAP' || item.id !== 'news.413' || !item.broadcast ||
        this.saved.story_choices?.['mission:JAP_GAW_war_exhaustion'] !== 'complete' ||
        !this.timelineByKey.get('mission:PRC_GAW_war_exhaustion')?.mission?.complete.calls.some(([id]) => id === item.id)) return [];
    // The installed mod has Japanese credits on its victory epilogue only.
    // Reuse the credits after a defeat without rendering the victory prose.
    const ending = this.timelineByKey.get('event:jap.1003');
    return [...new Set(ending?.options.flatMap((option) => option.credits ?? []) ?? [])];
  }

  private static readonly warsByMission: Record<string, string> = {
    PRC_taiwan_war_countdown: 'PRC_ongoing_taiwan_war', JAP_taiwan_war_mission: 'PRC_ongoing_taiwan_war',
    SOV_nato_war_deadline: 'SOV_nato_war_begins_global', second_nato_war_mission: 'second_nato_war_flag',
    third_euro_war_mission: 'FAF_third_euro_war_global', PRC_GAW_war_exhaustion: 'pan_asian_war_ongoing',
    JAP_GAW_war_exhaustion: 'pan_asian_war_ongoing',
  };

  private recordWarStart(item: TimelineItem, day: number) {
    if (!item.broadcast) return;
    const flags: Record<string, string> = {
      'news.115': 'USA_american_civil_war_global', 'news.139': 'PRC_ongoing_taiwan_war',
      'news.237': 'SOV_nato_war_begins_global', 'news.415': 'pan_asian_war_ongoing',
    };
    const missions = this.warNewsMissions[item.id.toLowerCase()] ?? [];
    const war = flags[item.id] ?? (missions.includes('second_nato_war_mission') ? 'second_nato_war_flag'
      : missions.includes('third_euro_war_mission') ? 'FAF_third_euro_war_global' : undefined);
    if (!war) return;
    const starts = this.saved.wartime_starts ??= {};
    starts[war] ??= day;
    const settled = Object.entries(App.warsByMission).some(([mission, flag]) => flag === war &&
      this.saved.read_story_ids.includes(`mission:${mission}`));
    if (!settled && !(war === 'USA_american_civil_war_global' && this.saved.country_flag_changes?.['global:USA_american_civil_war_over_global'])) {
      this.queueWartimeDevelopments(war, item, day, false);
    }
  }

  private queueWartimeDevelopments(war: string, source: TimelineItem, day: number, ending: boolean) {
    const started = this.saved.wartime_starts?.[war];
    if (started === undefined || day < started) return;
    const missionDay = Math.min(...Object.keys(App.warsByMission).filter((id) => App.warsByMission[id] === war).flatMap((id) => {
      const mission = this.timelineByKey.get(`mission:${id}`);
      const start = mission ? this.saved.mission_started?.[id] ?? this.foreignMissionStart(mission) : null;
      return mission?.mission && start !== null ? [start + mission.mission.days] : [];
    }));
    if (!ending && war !== 'USA_american_civil_war_global' && !Number.isFinite(missionDay)) return;
    // For a war without a dated outcome, offer territorial developments after
    // the opening bulletin. Other wars use the midpoint of their script timer.
    const anchor = ending ? day : Number.isFinite(missionDay) && missionDay > started
      ? started + Math.floor((missionDay - started) / 2) : day;
    const choices: NonNullable<TimelineItem['confirmation']>['choices'] = [];
    const seen = new Set<string>();
    for (const hook of this.wartimeHooks) {
      if (hook.war !== war || ending === (hook.kind === 'on_state_control_changed')) continue;
      for (const arm of hook.arms) {
        if (this.evaluateForCountry(arm.condition, this.saved.tag, anchor, true) === false) continue;
        const calls = arm.calls.filter(({ target, recipient }) => (this.timelineById.get(target.toLowerCase()) ?? []).some((item) =>
          !item.hidden && !this.saved.read_story_ids.includes(this.storyKey(item)) &&
          this.saved.option_due_days?.[this.storyKey(item)] === undefined &&
          (recipient === this.saved.tag || item.broadcast) && (!item.startup_trigger || item.startup_trigger === this.saved.tag)));
        if (!calls.length) continue;
        const key = calls.map(({ target, recipient }) => `${recipient}:${target}`).sort().join(',');
        if (seen.has(key)) continue;
        seen.add(key);
        const route: ConditionalRoute = { condition: arm.condition, trees: [], completions: [], flags: arm.flags, foreign_events: calls };
        const label = this.conditionalResultLabel(route, this.saved.tag);
        if (label) choices.push({ label, route, group: hook.id });
      }
    }
    if (!choices.length) return;
    this.queueConfirmation(source, `wartime:${war}:${ending}`, anchor, this.saved.tag, true,
      choices,
      { id: `wartime:${war}:${ending}`, title: ending ? '战事结算' : '战事进展', description: ending
        ? '勾选这场战争结算时发生的事件。' : '勾选这场战争期间发生的战况。', wartime: true });
  }

  private applyTreeChanges(trees: string[] = []) {
    for (const tree of trees) {
      if (this.countryChapters.some((chapter) => chapter.focus_tree_id === tree)) {
        this.saved.active_focus_tree = tree;
      }
    }
  }

  get conditionalFocusCount() {
    return this.routeFocuses.filter(
      (chapter) => !this.saved.read_focus_ids.includes(chapter.id) && !this.focusFlagsAllow(chapter),
    ).length;
  }

  get countryTimeline() {
    const tag = this.saved.tag.toUpperCase();
    if (this.countryTimelineCacheTag === tag) return this.countryTimelineCache;
    const focusSources = new Map(
      this.countryChapters.map((chapter) => [`focus:${chapter.id}`.toLowerCase(), chapter]),
    );
    const isInScope = (item: TimelineItem) =>
      (item.date_country ?? item.country_tag).toUpperCase() === tag ||
      (item.kind === 'news' && (item.broadcast || !item.country_tag)) ||
      item.linked_from.some((source) => {
        const normalized = source.toLocaleLowerCase();
        const sourceId = normalized.split(/[ (]/, 1)[0].replace(/^focus-select:/, 'focus:');
        return (
          (focusSources.has(sourceId) &&
            this.focusEventApplies(focusSources.get(sourceId), item, tag)) ||
          normalized.startsWith(`startup:${tag.toLowerCase()}`)
        );
      });
    const included = new Map<string, TimelineItem>();
    const externalTargets = new Set(
      (this.externalByTag.get(tag) ?? []).flatMap(({ events }) =>
        events.map(({ item }) => this.storyKey(item)),
      ),
    );
    const queue = this.timeline.filter(
      (item) => isInScope(item) || externalTargets.has(this.storyKey(item)) ||
        this.automaticForeignDay(item) !== null ||
        this.projectedStoryDays.has(`${tag}:${this.storyKey(item)}`),
    );
    for (const item of queue) included.set(`${item.kind}:${item.id}`, item);
    for (let index = 0; index < queue.length; index += 1) {
      const current = queue[index];
      for (const nextId of current.next_ids) {
        for (const next of this.timelineById.get(nextId.toLocaleLowerCase()) || []) {
          const currentSource = `${current.kind}:${current.id}`.toLocaleLowerCase();
          const followsScriptLink = next.linked_from.some((source) =>
            source.toLocaleLowerCase().startsWith(currentSource),
          );
          if (
            next.country_tag.toUpperCase() === tag ||
            (next.kind === 'news' && (next.broadcast || !next.country_tag)) ||
            followsScriptLink
          ) {
            const key = `${next.kind}:${next.id}`;
            if (!included.has(key)) {
              included.set(key, next);
              queue.push(next);
            }
          }
        }
      }
    }
    this.countryTimelineCache = [...included.values()];
    this.countryTimelineCacheTag = tag;
    return this.countryTimelineCache;
  }

  private focusEventApplies(
    source: { foreign_events?: ForeignEvent[] } | undefined,
    item: TimelineItem,
    tag: string,
  ) {
    const scoped = source?.foreign_events?.filter(
      (link) => link.target.toLowerCase() === item.id.toLowerCase(),
    );
    return !scoped?.length || scoped.some((link) => link.recipient === tag);
  }

  get externalDevelopments() {
    const read = new Set(this.saved.read_story_ids);
    return (this.externalByTag.get(this.saved.tag.toUpperCase()) ?? []).filter(({ events }) =>
      events.some(({ item }) => {
        const key = this.storyKey(item);
        return !read.has(key) && this.storyScheduleDay(item) === null;
      }),
    );
  }

  get availableSuperEvents(): SuperEvent[] {
    const scenes = new Map<string, SuperEvent>();
    for (const item of this.countryTimeline) {
      for (const scene of [item.super_event, ...item.options.map((option) => option.super_event)]) {
        if (!scene) continue;
        const existing = scenes.get(scene.id);
        if (!existing || (!existing.audio && scene.audio)) scenes.set(scene.id, scene);
      }
    }
    for (const focus of this.routeFocuses) {
      if (focus.super_event && !scenes.has(focus.super_event.id)) {
        scenes.set(focus.super_event.id, focus.super_event);
      }
    }
    return [...scenes.values()].sort((a, b) => a.title.localeCompare(b.title));
  }

  previewSuperEvent(scene: SuperEvent) {
    this.openSuperEvent(scene);
  }

  get openingEvents() {
    return this.dueStories;
  }

  get scheduledStories() {
    this.refreshScheduleCache();
    if (this.scheduledStoryCache) return this.scheduledStoryCache;
    const stories = new Set(this.countryTimeline);
    for (const { item } of Object.values(this.saved.confirmations ?? {})) stories.add(item);
    for (const id of Object.keys(this.saved.decision_started ?? {})) {
      const decision = this.timelineByKey.get(`decision:${id}`);
      if (decision?.decision_end_music) stories.add({ ...decision, kind: 'decision-end' });
    }
    for (const id of ['PRC_taiwan_war_countdown', 'SOV_nato_war_deadline', 'PRC_GAW_war_exhaustion']) {
      const mission = this.timelineByKey.get(`mission:${id}`);
      if (mission && this.foreignMissionStart(mission) !== null) stories.add(mission);
    }
    for (const key of Object.keys(this.saved.option_due_days ?? {})) {
      const item = this.timelineByKey.get(key);
      if (item) stories.add(item);
    }
    for (const id of Object.keys(this.saved.mission_started ?? {})) {
      const item = this.timelineByKey.get(`mission:${id}`);
      if (item?.mission?.shared_resolution) stories.add(item);
    }
    const entries = [...stories]
      .filter(
        (item) =>
          (!item.hidden || this.hiddenStoryHasEffects(item)) &&
          ['event', 'news', 'mission', 'confirmation', 'decision-end'].includes(item.kind) &&
          this.checksPass(item.trigger_checks ?? []) &&
          !this.skipTaiwanFallback(item),
      )
      .map((item) => ({ item, day: this.calculateStoryDay(item, new Set()) }))
      .filter((entry): entry is { item: TimelineItem; day: number } => entry.day !== null)
      .sort(
        ({ item: a, day: aDay }, { item: b, day: bDay }) =>
          aDay - bDay ||
          Number(b.kind === 'confirmation') - Number(a.kind === 'confirmation') ||
          Number(this.storyKey(b) === 'news:news.139' && this.automaticForeignDay(b) !== null) -
            Number(this.storyKey(a) === 'news:news.139' && this.automaticForeignDay(a) !== null) ||
          Number(a.kind === 'mission') - Number(b.kind === 'mission') ||
          Number(this.isImmediateParent(b, a)) - Number(this.isImmediateParent(a, b)) ||
          Number(!!b.hidden) - Number(!!a.hidden) ||
          a.title.localeCompare(b.title),
      );
    this.scheduledStoryCache = entries.map(({ item }) => item);
    return this.scheduledStoryCache;
  }

  private hiddenStoryHasEffects(item: TimelineItem) {
    return !!(item.flag_changes?.length || item.focus_tree_changes?.length || item.focus_completions?.length ||
      item.mission_activations?.length || item.conditional_routes?.length || item.music?.length || item.super_event);
  }

  private isImmediateParent(parent: TimelineItem, child: TimelineItem) {
    const key = this.storyKey(parent);
    return this.hasDirectCall(parent.immediate_calls, child) &&
      child.linked_from.some((source) => source === key || source.startsWith(key + ' ('));
  }

  private restoreTaiwanLoss() {
    if (this.saved.tag !== 'PRC' || !this.saved.read_story_ids.includes('event:china.24')) return;
    const loss = this.timelineByKey.get('event:china.24');
    const root = this.focusById.get('PRC_shameful_display');
    if (!loss || !root || !this.activeFocusTreeIds.has(root.focus_tree_id)) return;
    const day = this.storyScheduleDay(loss);
    if (day === null) return;
    this.applyFlagChanges(loss.flag_changes);
    this.applyFocusCompletions(loss.focus_completions, day);
    for (const id of ['china.85', 'china.148', 'china.150', 'china.37']) {
      const key = `event:${id}`;
      if (!this.saved.read_story_ids.includes(key)) continue;
      const story = this.timelineByKey.get(key);
      const chosen = story?.options.find((option) => option.id === this.saved.story_choices?.[key]);
      this.applyFlagChanges(story?.flag_changes);
      this.applyFlagChanges(chosen?.flag_changes);
    }
    const salvage = this.focusById.get('PRC_salvage_the_situation');
    const salvageDay = salvage && this.saved.focus_completion_days[salvage.id];
    if (salvageDay !== undefined && salvage) {
      this.scheduleScriptCalls(salvage.selection_calls, salvageDay - (salvage.duration_days || 70));
    }
    const session = this.focusById.get('PRC_begin_the_session_of_cpc');
    const sessionDay = session && this.saved.focus_completion_days[session.id];
    if (sessionDay !== undefined && session) this.applyConditionalRoutes(session.conditional_routes, sessionDay);
    this.persist();
  }

  private activateStoryMissions(item: TimelineItem, day: number) {
    this.activateMissions(item.mission_activations, day);
    if (item.broadcast) this.activateMissions(this.warNewsMissions[item.id.toLowerCase()], day);
    if (item.id === this.greatAsianWarStart?.target && this.saved.tag === this.greatAsianWarStart.recipient) {
      this.activateMissions(['JAP_GAW_war_exhaustion'], day);
    }
    this.recordWarStart(item, day);
  }

  private activateMissions(ids: string[] = [], day: number) {
    for (const id of ids) {
      const mission = this.timelineByKey.get(`mission:${id}`);
      if (!mission?.mission || (!mission.mission.shared_resolution && mission.country_tag && mission.country_tag !== this.saved.tag)) continue;
      const starts = (this.saved.mission_started ??= {});
      starts[id] ??= day;
    }
  }

  private adjustMissionDeadlines(changes: [string, number][] = []) {
    for (const [id, days] of changes) {
      if (this.saved.mission_started?.[id] === undefined ||
          this.saved.read_story_ids.includes(`mission:${id}`)) continue;
      const adjustments = (this.saved.mission_deadline_adjustments ??= {});
      adjustments[id] = (adjustments[id] ?? 0) + days;
    }
  }

  private restoreMissions() {
    for (const [id, day] of Object.entries(this.saved.focus_completion_days)) {
      this.activateMissions(this.focusById.get(id)?.mission_activations, day);
    }
    for (const key of this.saved.read_story_ids) {
      const item = this.timelineByKey.get(key);
      if (!item) continue;
      const scheduled = this.storyScheduleDay(item);
      if (scheduled !== null) this.recordWarStart(item, scheduled);
      const chosen = item.options.find(({ id }) => id === this.saved.story_choices?.[key]);
      if (!item.mission_activations?.length && !chosen?.mission_activations?.length &&
          !(item.broadcast && this.warNewsMissions[item.id.toLowerCase()]?.length) &&
          item.id !== this.greatAsianWarStart?.target) continue;
      const day = scheduled;
      if (day === null) continue;
      this.activateStoryMissions(item, day);
      this.activateMissions(chosen?.mission_activations, day);
    }
  }

  private foreignMissionEffect(effect: MissionEffect): MissionEffect {
    return {
      music: effect.music,
      calls: effect.calls.filter(([id]) =>
        this.timelineById.get(id.toLowerCase())?.some((item) => item.broadcast),
      ),
      foreign_events: effect.foreign_events,
      conditional_routes: effect.conditional_routes,
      flags: effect.flags.filter(({ flag }) => flag.startsWith('global:')),
      trees: [],
      completions: [],
    };
  }

  private warNewsMissions: Record<string, string[]> = {};

  private foreignMissionStart(item: TimelineItem) {
    const tag = this.saved.tag;
    if (!tag || !item.mission) return null;
    if (item.id === 'PRC_taiwan_war_countdown' && tag !== 'PRC' && tag !== 'JAP') {
      const news = this.timelineByKey.get('news:news.139');
      const focus = this.focusById.get('PRC_finish_what_mao_started');
      if (news && focus?.mission_activations?.includes(item.id)) {
        return this.automaticForeignDay(news) ?? this.projectedForeignDay(news);
      }
    }
    if (item.id === 'SOV_nato_war_deadline' && tag !== 'SOV' && tag !== 'UKR') {
      const intervention = this.timelineByKey.get('mission:NATO_intervention_timer')?.mission;
      if (intervention?.timeout.activated_missions?.some(([owner, id]) => owner === 'SOV' && id === item.id)) {
        return this.europeanWarDay;
      }
    }
    if (item.id === 'PRC_GAW_war_exhaustion' && tag !== 'PRC' && tag !== 'JAP' && this.greatAsianWarNews) {
      const start = this.calendarDay('2028.1.1');
      const delay = this.parseStoryDelay(this.greatAsianWarNews[1]);
      if (start !== null && delay !== null) return start + delay;
    }
    return null;
  }

  get currentForeignMission() {
    return !!this.current && 'kind' in this.current && this.foreignMissionStart(this.current) !== null;
  }

  get currentForeignMissionTitle() {
    if (this.current?.id === 'PRC_taiwan_war_countdown') return '第一次台海战争的结局';
    return this.current && this.current.id === 'SOV_nato_war_deadline' ? '第一次欧洲战争的结局' : '大东亚战争的结局';
  }

  private scheduleScopedCalls(links: ForeignEvent[] = [], day: number, projected = false) {
    for (const { recipient, target, delay } of links) {
      for (const item of this.timelineById.get(target.toLowerCase()) ?? []) {
        if (recipient === this.saved.tag || item.broadcast) this.scheduleEvent(item, delay, day, projected);
      }
    }
  }

  private missionChoices(item: TimelineItem): MissionResolution[] {
    const mission = item.mission;
    if (!mission || this.automaticallyTimesOut(item)) return [];
    const foreign = this.foreignMissionStart(item) !== null;
    if (mission.shared_resolution) {
      const second = item.id === 'second_nato_war_mission';
      return [
        { id: 'complete', label: second ? '俄罗斯一方获胜' : '法国一方获胜', effect: mission.complete },
        { id: 'cancel', label: second ? '欧洲一方获胜' : '德国一方获胜', effect: mission.cancel },
      ];
    }
    if (item.id === 'USA_countdown_day_mission' && this.saved.tag === 'USA') {
      // Several power-balance paths all certify one of the same two candidates.
      // Resolve the certification once, then follow its actual inauguration and civil-war timer.
      const calls = this.conditionalLocalCalls(mission.timeout.conditional_routes);
      const results = ['usa.14', 'usa.128'].map((id) => ({
        id, story: this.timelineByKey.get(`event:${id}`), call: calls.find(([target]) => target === id),
      }));
      if (results.every(({ story, call }) => !!story && !!call)) return results.map(({ id, story, call }) => ({
        id, label: story!.title, effect: { ...mission.timeout, calls: [call!], conditional_routes: [] },
      }));
    }
    if (item.id === 'SOV_nato_war_deadline' && mission.cancel.calls.some(([id]) => id === 'news.204') &&
        mission.complete.flags.some(({ flag, present }) => flag === 'global:SOV_nato_war_over_global' && present)) {
      const knownVictories = [
        { id: 'news.191', label: '俄罗斯获胜，由俄共领导', flag: 'SOV_cprf_won' },
        { id: 'news.216', label: '俄罗斯获胜，由梅德韦杰夫领导', flag: 'SOV_medvedev_won' },
        { id: 'news.255', label: '俄罗斯获胜，由日里诺夫斯基领导', flag: 'SOV_ldpr_won' },
      ].flatMap(({ id, label, flag }) => {
        const outcome = this.timelineByKey.get(`news:${id}`);
        const source = outcome?.linked_from.find((link) => link.startsWith(`mission:${item.id} (`));
        if (!outcome?.broadcast || !source) return [];
        const delay = source.slice(source.indexOf('(') + 1, -1);
        const effect = this.foreignMissionEffect(mission.complete);
        return [{ id, label, effect: { ...effect, calls: [...effect.calls, [id, delay] as [string, string]] },
          flags: [{ flag: `global:${flag}`, present: true }] }];
      });
      if (foreign && knownVictories.length === 3) {
        return [...knownVictories, { id: 'cancel', label: '北约一方达成战争目标', effect: this.foreignMissionEffect(mission.cancel) }];
      }
      return [
        { id: 'complete', label: '俄罗斯一方达成战争目标', effect: mission.complete },
        { id: 'cancel', label: '北约一方达成战争目标', effect: mission.cancel },
      ].map((choice) => foreign ? { ...choice, effect: this.foreignMissionEffect(choice.effect) } : choice);
    }
    if (item.id === 'SOV_countdown_day_mission' && this.saved.tag === 'SOV') {
      const results = [
        ['russia.46', 'SOV_cprf_won'],
        ['russia.47', 'SOV_ldpr_won'],
        ['russia.49', 'SOV_medvedev_won'],
        ['russia.50', 'SOV_srzp_won'],
      ];
      if (results.some(([id, flag]) => !mission.timeout.calls.some(([target]) => target === id) ||
          !this.timelineByKey.get(`event:${id}`)?.trigger_checks?.some(
            (check) => check.flag === `global:${flag}` && check.present,
          ))) return [];
      if (results.some(([id]) => this.saved.read_story_ids.includes(`event:${id}`))) return [];
      return results.map(([id, flag]) => ({
        id,
        label: this.timelineByKey.get(`event:${id}`)!.title,
        flags: results.map(([, resultFlag]) => ({ flag: `global:${resultFlag}`, present: resultFlag === flag })),
        effect: { ...mission.timeout, calls: mission.timeout.calls.filter(([target]) => target === id) },
      }));
    }
    if (item.id === 'JAP_election_mission' && this.saved.tag === 'JAP') {
      const results = [
        { id: 'jap.17', flag: 'global:JAP_ldp_won', label: '自民党赢得选举' },
        { id: 'jap.18', flag: 'global:JAP_cc_won', label: '市民联合赢得选举' },
      ];
      if (results.every(({ id, flag }) => mission.timeout.calls.some(([target]) => target === id) &&
          this.timelineByKey.get(`event:${id}`)?.trigger_checks?.some(
            (check) => check.flag === flag && check.present,
          ))) {
        if (results.some(({ id }) => this.saved.read_story_ids.includes(`event:${id}`))) return [];
        return results.map(({ id, flag, label }) => ({
          id, label, flags: [{ flag, present: true }],
          effect: { ...mission.timeout, calls: mission.timeout.calls.filter(([target]) => target === id) },
        }));
      }
      return [];
    }
    if (['PRC_taiwan_war_countdown', 'JAP_taiwan_war_mission'].includes(item.id)) {
      const china = this.timelineByKey.get('mission:PRC_taiwan_war_countdown')?.mission;
      if (!china || !china.complete.calls.some(([id]) => id === 'china.23') ||
          !china.timeout.calls.some(([id]) => id === 'china.24') ||
          !china.complete.foreign_events.some(({ recipient, target }) => recipient === 'JAP' && target === 'jap.10') ||
          !china.timeout.foreign_events.some(({ recipient, target }) => recipient === 'JAP' && target === 'jap.9')) return [];
      const japan = this.saved.tag === 'JAP';
      if ((japan ? ['jap.9', 'jap.10'] : ['china.23', 'china.24']).some(
        (id) => this.saved.read_story_ids.includes(`event:${id}`),
      )) return [];
      return [
        { id: 'complete', label: '期限内，中国大陆控制任务要求的全部地区', effect: china.complete },
        { id: 'timeout', label: '期限已到，中国大陆尚未控制全部目标地区', effect: china.timeout },
      ].map((choice) => japan || foreign ? { ...choice, effect: this.foreignMissionEffect(choice.effect) } : choice);
    }
    if (item.id === 'PRC_GAW_war_exhaustion' && (this.saved.tag === 'PRC' || foreign) &&
        mission.complete.calls.some(([id]) => id === 'news.413') &&
        mission.cancel.calls.some(([id]) => id === 'news.414')) {
      return [
        { id: 'complete', label: '中国一方达成战争目标', effect: mission.complete },
        { id: 'cancel', label: '日本一方获胜，中国触发投降结算', effect: mission.cancel },
      ].map((choice) => foreign ? { ...choice, effect: this.foreignMissionEffect(choice.effect) } : choice);
    }
    if (item.id === 'JAP_GAW_war_exhaustion' && this.saved.tag === 'JAP' && this.greatAsianWarStart) {
      const china = this.timelineByKey.get('mission:PRC_GAW_war_exhaustion')?.mission;
      if (!china?.cancel.foreign_events.some(({ recipient, target }) =>
        recipient === 'JAP' && target === 'jap.1003',
      )) return [];
      return [
        { id: 'cancel', label: '日本一方获胜，中国触发投降结算', effect: this.foreignMissionEffect(china.cancel) },
        { id: 'complete', label: '中国一方达成战争目标', effect: this.foreignMissionEffect(china.complete) },
      ];
    }
    const choices: MissionResolution[] = [];
    if (this.missionHasResult(mission.complete)) choices.push({ id: 'complete', label: '达成任务目标', effect: mission.complete });
    if (this.missionHasResult(mission.timeout)) choices.push({ id: 'timeout', label: '期限内未达成目标', effect: mission.timeout });
    if (this.missionHasResult(mission.cancel)) choices.push({ id: 'cancel', label: '任务中止', effect: mission.cancel });
    return choices;
  }

  private missionHasResult(effect: MissionEffect) {
    return !!(effect.calls.length || this.hasLocalOrBroadcastCalls(effect.foreign_events) ||
      effect.flags.some(({ flag }) => this.focusConditionFlags.has(flag)) ||
      effect.trees.length || effect.completions.length || effect.music?.length ||
      this.hasConditionalStoryCalls(effect.conditional_routes));
  }

  private conditionalLocalCalls(routes: ConditionalRoutes = []): [string, string][] {
    return routes.flatMap((chain) => chain.flatMap((route) => [
      ...(route.owner && route.owner !== this.saved.tag ? [] : route.calls ?? []),
      ...this.conditionalLocalCalls(route.nested),
    ]));
  }

  private automaticallyTimesOut(item: TimelineItem) {
    if (item.automatic_timeout) return true;
    const mission = item.mission;
    if (!mission || mission.shared_resolution ||
        ['JAP_election_mission', 'SOV_countdown_day_mission', 'USA_countdown_day_mission', 'PRC_taiwan_war_countdown',
          'JAP_taiwan_war_mission', 'JAP_GAW_war_exhaustion', 'PRC_GAW_war_exhaustion', 'SOV_nato_war_deadline'].includes(item.id)) return false;
    return !this.missionHasResult(mission.complete) && !this.missionHasResult(mission.cancel) && this.missionHasResult(mission.timeout);
  }

  get currentMissionChoices() {
    if (!this.current || !('kind' in this.current) || this.current.kind !== 'mission' ||
        this.saved.read_story_ids.includes(this.storyKey(this.current))) return [];
    const day = this.storyScheduleDay(this.current);
    if (day === null || day > this.routeDay) return [];
    if (this.nextDueStory !== this.current) return [];
    return this.missionChoices(this.current);
  }

  get currentConfirmation() {
    return this.current && 'kind' in this.current && !this.saved.read_story_ids.includes(this.storyKey(this.current))
      ? this.current.confirmation : undefined;
  }

  wartimeChoices = new Set<number>();

  setWartimeChoice(index: number, checked: boolean) {
    const choices = this.currentConfirmation?.choices;
    if (!choices?.[index]) return;
    const group = choices[index].group;
    if (checked) {
      for (const previous of this.wartimeChoices) {
        if (group && choices[previous].group === group) this.wartimeChoices.delete(previous);
      }
      this.wartimeChoices.add(index);
    } else this.wartimeChoices.delete(index);
  }

  resolveConfirmation(index: number) {
    this.resolveConfirmationChoices([index]);
  }

  resolveWartimeChoices() {
    if (this.currentConfirmation?.wartime) this.resolveConfirmationChoices([...this.wartimeChoices]);
  }

  private resolveConfirmationChoices(indices: number[]) {
    const current = this.current;
    const confirmation = this.currentConfirmation;
    if (this.storyPreview || !current || !('kind' in current) || !confirmation || indices.some((index) => !confirmation.choices[index])) return;
    const key = this.storyKey(current);
    const day = this.saved.confirmations?.[key]?.day;
    if (day === undefined || day > this.routeDay || this.nextDueStory !== current) return;
    this.checkpointRoute('event');
    this.saved.read_story_ids = [...this.saved.read_story_ids, key];
    this.saved.story_choices = { ...this.saved.story_choices, [key]: indices.join(',') };
    let scene: SuperEvent | null | undefined;
    for (const index of indices) {
      const choice = confirmation.choices[index];
      this.applyFlagChanges(choice.flags);
      for (const route of choice.routes ?? (choice.route ? [choice.route] : [])) {
        this.applyConditionalArm(route, day, confirmation.owner, confirmation.projected, current, String(index));
      }
      if (choice.event) this.scheduleScriptCalls([[choice.event, '']], day, confirmation.projected);
      const decision = choice.decision ? this.timelineByKey.get(`decision:${choice.decision}`) : undefined;
      if (decision) scene = this.activateDecision(decision, day);
    }
    this.persist();
    this.openSuperEvent(scene, () => void this.continueRoute());
  }

  resolveMission(id: string) {
    const mission = this.current;
    const choice = this.currentMissionChoices.find((option) => option.id === id);
    if (!mission || !('kind' in mission) || !choice) return;
    const deadline = this.storyScheduleDay(mission);
    if (deadline === null) return;
    if (this.nextDueStory !== mission) return;
    this.checkpointRoute('event');
    const war = App.warsByMission[mission.id];
    if (war) {
      const starts = this.saved.wartime_starts ??= {};
      starts[war] ??= this.saved.mission_started?.[mission.id] ?? this.foreignMissionStart(mission) ?? deadline;
      this.queueWartimeDevelopments(war, mission, deadline, true);
    }
    const effect = choice.effect;
    const foreign = this.foreignMissionStart(mission) !== null;
    const projected = foreign || !!mission.mission?.shared_resolution;
    this.applyFlagChanges(choice.flags);
    const owner = mission.id === 'JAP_GAW_war_exhaustion' ? 'PRC' : mission.country_tag || this.saved.tag;
    const scoped = mission.mission?.shared_resolution && owner !== this.saved.tag;
    this.applyFlagChanges(scoped ? effect.flags.filter(({ flag }) => flag.startsWith('global:')) : effect.flags);
    if (mission.id === 'SOV_nato_war_deadline' && effect.flags.some(({ flag, present }) =>
      flag === 'global:SOV_nato_war_over_global' && present)) {
      this.applyFlagChanges([{ flag: 'global:nato_nato_won_nato_war', present: id === 'cancel' }]);
    }
    this.playScriptedMusic(effect.music, owner, deadline);
    if (!scoped) {
      this.applyTreeChanges(effect.trees);
      this.applyFocusCompletions(effect.completions, deadline);
    } else {
      this.applyFocusCompletions(effect.completions.filter(({ tag }) => tag === this.saved.tag), deadline);
    }
    if (mission.id === 'JAP_taiwan_war_mission' || (foreign && mission.id === 'PRC_taiwan_war_countdown')) {
      const counterpart = this.timelineByKey.get('mission:PRC_taiwan_war_countdown')?.mission;
      const chosen = id === 'complete' ? counterpart?.complete : counterpart?.timeout;
      if (chosen) this.scheduleWorldConsequences(chosen, 'PRC', deadline);
    } else if (foreign && mission.id === 'SOV_nato_war_deadline' && id !== 'cancel') {
      const full = mission.mission?.complete;
      if (full) this.scheduleWorldConsequences(full, 'SOV', deadline);
    }
    if (scoped) this.scheduleScopedCalls(effect.calls.map(([target, delay]) => ({ recipient: owner, target, delay })), deadline, projected);
    else this.scheduleScriptCalls(effect.calls, deadline, projected);
    this.scheduleScopedCalls(effect.foreign_events, deadline, projected);
    const routes = effect.conditional_routes;
    // TFR writes the alternative Russian governments as separate if blocks.
    // They describe mutually exclusive routes, so one selection resolves them together.
    const outcomes = mission.id === 'second_nato_war_mission' && routes?.every((chain) => chain.length === 1)
      ? [routes.flat()] : routes;
    this.applyConditionalRoutes(outcomes, deadline, owner, projected, mission, id);
    const key = this.storyKey(mission);
    this.saved.read_story_ids = [...this.saved.read_story_ids, key];
    this.saved.story_choices = { ...this.saved.story_choices, [key]: id };
    this.persist();
    void this.continueRoute();
  }

  private skipTaiwanFallback(item: TimelineItem) {
    if (this.saved.tag !== 'PRC' || item.id !== 'china.124' ||
        !/(?:^|[\\/])TFR_events_PRC\.txt$/i.test(item.source)) return false;
    const flags = this.saved.country_flag_changes;
    return this.saved.read_focus_ids.includes('PRC_finish_what_mao_started') ||
      ['ongoing_taiwan_war', 'taiwan_war_won', 'taiwan_war_lost'].some(
        (flag) => flags?.[`global:PRC_${flag}`],
      );
  }

  get dueStories() {
    const read = new Set(this.saved.read_story_ids || []);
    const day = this.routeDay;
    const deferEnding = this.shouldDeferEnding();
    return this.scheduledStories.filter(
      (item) =>
        !read.has(this.storyKey(item)) && (!deferEnding || !this.hasEnding(item)) &&
        (this.calculateStoryDay(item, new Set()) ?? Infinity) <= day,
    );
  }

  get nextDueStory() {
    return this.dueStories[0];
  }

  get upcomingStories() {
    const read = new Set(this.saved.read_story_ids);
    const day = this.routeDay;
    const deferEnding = this.shouldDeferEnding();
    return this.scheduledStories
      .filter(
        (item) =>
          !read.has(this.storyKey(item)) && (!deferEnding || !this.hasEnding(item)) &&
          (this.calculateStoryDay(item, new Set()) ?? -1) > day,
      )
      .slice(0, 4);
  }

  get nextFutureStory() {
    return this.upcomingStories[0];
  }

  get currentIsScheduledStory() {
    if (!this.current || !('kind' in this.current)) return false;
    const day = this.storyScheduleDay(this.current);
    return day !== null && day <= this.routeDay;
  }

  get currentScheduledDay() {
    return this.current && 'kind' in this.current ? this.storyScheduleDay(this.current) : null;
  }

  get focusSideStories() {
    if (!this.current || 'kind' in this.current) return [];
    const focus = this.current;
    const source = `focus:${focus.id}`;
    return this.countryTimeline
      .filter(
        (item) =>
          !this.saved.read_story_ids.includes(this.storyKey(item)) &&
          this.focusEventApplies(focus, item, this.saved.tag) &&
          this.canFollowClue(item, this.hasDirectCall(focus.event_calls, item)) &&
          item.linked_from.some(
            (relation) => relation === source || relation.startsWith(source + ' ('),
          ) &&
          (!['event', 'news'].includes(item.kind) || this.storyScheduleDay(item) === null),
      )
      .sort((a, b) => {
        const delay = (item: TimelineItem) => {
          const relation = item.linked_from.find((link) => link.startsWith(source)) || '';
          const days = relation.match(/days\s+(\d+)/);
          return days ? Number(days[1]) : 0;
        };
        return delay(a) - delay(b) || a.id.localeCompare(b.id);
      });
  }

  private storyKey(item: TimelineItem) {
    return `${item.kind}:${item.id}`;
  }

  calendarLabel(day: number | null): string {
    if (day === null) return '日期未确定';
    return this.formatGameDate(this.startDate, day) ?? `第 ${day} 天`;
  }

  formatGameDate(raw: string | null, offset = 0): string | null {
    const timestamp = this.gameDateMillis(raw);
    if (timestamp === null) return null;
    const date = new Date(timestamp);
    date.setUTCDate(date.getUTCDate() + offset);
    const locales = {
      english: 'en-GB',
      simp_chinese: 'zh-CN',
      traditional_chinese: 'zh-TW',
      japanese: 'ja-JP',
    };
    return new Intl.DateTimeFormat(locales[this.saved.language], {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      timeZone: 'UTC',
    }).format(date);
  }

  private gameDateMillis(raw: string | null): number | null {
    if (!raw) return null;
    const match = /^(\d{1,4})\.(\d{1,2})\.(\d{1,2})(?:\.\d{1,2})?$/.exec(raw);
    if (!match) return null;
    const [, year, month, day] = match.map(Number);
    const date = new Date(0);
    date.setUTCFullYear(year, month - 1, day);
    if (date.getUTCMonth() !== month - 1 || date.getUTCDate() !== day) return null;
    return date.getTime();
  }

  storyDay(item: TimelineItem | null | undefined) {
    if (!item) return 0;
    return this.storyScheduleDay(item) ?? item.start_day ?? 0;
  }

  private scheduledStoryCache?: TimelineItem[];

  private refreshScheduleCache() {
    const focusSignature = JSON.stringify([
      this.saved.tag,
      this.saved.active_focus_tree,
      this.saved.option_due_days,
      this.saved.decision_started,
      this.saved.country_flag_changes,
      this.saved.mission_started,
      this.saved.mission_deadline_adjustments,
      Object.keys(this.saved.confirmations ?? {}),
      this.saved.read_story_ids,
      this.saved.read_focus_ids,
      this.saved.focus_completion_days,
    ]);
    if (focusSignature !== this.scheduleCacheFocusSignature) {
      this.scheduleCache.clear();
      this.scheduledStoryCache = undefined;
      this.scheduleCacheFocusSignature = focusSignature;
    }
  }

  private storyScheduleDay(item: TimelineItem): number | null {
    this.refreshScheduleCache();
    return this.calculateStoryDay(item, new Set());
  }

  private calculateStoryDay(item: TimelineItem, visiting: Set<string>): number | null {
    if (item.kind === 'confirmation') return this.saved.confirmations?.[this.storyKey(item)]?.day ?? null;
    if (item.kind === 'decision-end') {
      const start = this.saved.decision_started?.[item.id];
      return start === undefined || !item.decision_end_music ? null : start + item.decision_end_music.days;
    }
    if (item.kind === 'mission') {
      const start = this.saved.mission_started?.[item.id] ?? this.foreignMissionStart(item);
      if (!item.mission || start === null ||
          this.saved.read_story_ids.includes(this.storyKey(item)) ||
          (!this.automaticallyTimesOut(item) && !this.missionChoices(item).length)) return null;
      const deadline = start + Math.max(0, item.mission.days + (this.saved.mission_deadline_adjustments?.[item.id] ?? 0));
      return item === this.activeWarMission ? this.warResolutionDay(start, deadline) : deadline;
    }
    const key = this.storyKey(item);
    if (visiting.has(key)) return null;
    if (this.scheduleCache.has(key)) return this.scheduleCache.get(key) ?? null;
    visiting.add(key);
    const optionDay = this.saved.option_due_days?.[key];
    const automaticDay = this.automaticForeignDay(item);
    if (optionDay !== undefined || automaticDay !== null) {
      const day = Math.min(optionDay ?? Infinity, automaticDay ?? Infinity);
      this.scheduleCache.set(key, day);
      return day;
    }
    if (item.date_after) return this.autonomousStoryDay(item, this.saved.tag, this.saved.focus_completion_days);
    if (
      item.date &&
      this.startDate &&
      (!item.date_country || item.date_country === this.saved.tag)
    ) {
      const date = this.gameDateMillis(item.date);
      const start = this.gameDateMillis(this.startDate);
      if (date !== null && start !== null) {
        return Math.max(0, Math.round((date - start) / 86_400_000));
      }
    }
    const startupLinks = item.linked_from.filter((source) => source.startsWith('startup:'));
    const startupDays = this.startupDays(item, this.saved.tag);
    if (startupDays.length) {
      const day = Math.min(...startupDays);
      this.scheduleCache.set(key, day);
      return day;
    }
    if (!startupLinks.length && item.start_day != null) return item.start_day;
    if (!['event', 'news'].includes(item.kind)) {
      this.scheduleCache.set(key, null);
      return null;
    }

    const possibleDays: number[] = [];
    for (const source of item.linked_from) {
      if (source.includes('/option:')) continue;

      const focusMatch = source.match(/^focus:([^\s(]+)(?:\s+\((.*?)\))?$/);
      if (focusMatch) {
        const completedDay = this.saved.focus_completion_days?.[focusMatch[1]];
        const focus = this.localFocus(focusMatch[1]) ?? this.focusById.get(focusMatch[1]);
        if (!this.focusEventApplies(focus, item, this.saved.tag)) continue;
        const rawDelay = focusMatch[2] || '';
        if (!this.hasDirectCall(focus?.event_calls, item, rawDelay) &&
            !focus?.foreign_events?.some((link) =>
              link.recipient === this.saved.tag && link.target.toLowerCase() === item.id.toLowerCase() &&
              link.delay === rawDelay,
            )) continue;
        const delay = this.parseStoryDelay(rawDelay);
        if (completedDay !== undefined && delay !== null) possibleDays.push(completedDay + delay);
        continue;
      }

      const parentMatch = source.match(/^(event|news|decision|mission):([^/ (]+)(?:\s+\((.*?)\))?/);
      if (!parentMatch) continue;
      const parent = this.timelineByKey.get(`${parentMatch[1]}:${parentMatch[2]}`);
      if (!parent) continue;
      if (parent.kind === 'decision') {
        const start = this.saved.decision_started?.[parent.id];
        if (start === undefined) continue;
        for (const call of parent.decision_guarded_calls ?? []) {
          if (!call.deferred || call.target.toLowerCase() !== item.id.toLowerCase() ||
              !this.checksPass(call.checks)) continue;
          const delay = this.parseStoryDelay(call.delay);
          if (delay !== null) possibleDays.push(start + delay);
        }
        continue;
      }
      if (!['event', 'news'].includes(parent.kind)) continue;
      const scoped = parent.foreign_events?.filter(
        (link) => link.target.toLowerCase() === item.id.toLowerCase(),
      );
      if (scoped?.length) {
        const ownerStartup = this.startupDays(parent, parent.country_tag);
        const parentDay = ownerStartup.length
          ? Math.min(...ownerStartup)
          : this.calculateStoryDay(parent, new Set(visiting));
        for (const link of scoped) {
          if (link.recipient !== this.saved.tag) continue;
          const delay = this.parseStoryDelay(link.delay);
          if (parentDay !== null && delay !== null) possibleDays.push(parentDay + delay);
        }
      } else {
        if (!this.hasDirectCall(parent.immediate_calls, item, parentMatch[3] || '')) continue;
        const directBroadcast = item.broadcast &&
          parent.broadcast_calls?.some(([target, delay]) =>
            target.toLowerCase() === item.id.toLowerCase() && delay === (parentMatch[3] || ''),
          );
        const ownerStartup = directBroadcast ? this.startupDays(parent, parent.country_tag) : [];
        const parentDay = ownerStartup.length
          ? Math.min(...ownerStartup)
          : this.calculateStoryDay(parent, new Set(visiting));
        const delay = this.parseStoryDelay(parentMatch[3] || '');
        if (parentDay !== null && delay !== null) possibleDays.push(parentDay + delay);
      }
    }
    const day = possibleDays.length ? Math.min(...possibleDays) : this.projectedForeignDay(item);
    if (day !== null) this.scheduleCache.set(key, day);
    return day;
  }

  private warResolutionDay(start: number, deadline: number): number | null {
    const focuses = this.warFocusIds;
    if (this.availableFocuses.some(({ id }) => focuses.has(id))) return null;
    const read = new Set(this.saved.read_story_ids);
    const opening = this.saved.tag === 'PRC' ? 'china.1008' : 'jap.1002';
    const openingStory = this.timelineByKey.get(`event:${opening}`);
    if (openingStory && (!read.has(`event:${opening}`) ||
        (openingStory.options.length && !this.saved.story_choices?.[`event:${opening}`] &&
          !this.saved.read_focus_ids.includes(this.saved.tag === 'PRC' ? 'PRC_chinas_greatest_hour'
            : 'JAP_second_war_in_greater_east_asia')))) return null;
    const keys = new Set([`event:${opening}`, 'news:news.415']);
    const focusSources = new Set([...focuses].map((id) => `focus:${id}`));
    const queue = this.countryTimeline.filter((item) => ['event', 'news'].includes(item.kind) &&
      (keys.has(this.storyKey(item)) || item.linked_from.some(
        (source) => focusSources.has(source.split(' (')[0]),
      )));
    const visited = new Set(queue.map((item) => this.storyKey(item)));
    let day = Math.max(deadline, ...[...focuses].map((id) => this.saved.focus_completion_days[id] ?? start));
    for (let index = 0; index < queue.length; index++) {
      const item = queue[index];
      const due = this.calculateStoryDay(item, new Set());
      if (due === null || due < start || !this.checksPass(item.trigger_checks ?? [])) continue;
      day = Math.max(day, due);
      for (const id of item.next_ids) {
        for (const next of this.timelineById.get(id.toLowerCase()) ?? []) {
          const key = this.storyKey(next);
          if (visited.has(key) || !['event', 'news'].includes(next.kind)) continue;
          visited.add(key);
          queue.push(next);
        }
      }
    }
    return day;
  }

  private hasDirectCall(calls: [string, string][] = [], item: TimelineItem, delay?: string) {
    return calls.some(([target, rawDelay]) =>
      target.toLowerCase() === item.id.toLowerCase() && (delay === undefined || delay === rawDelay),
    );
  }

  private startupDays(item: TimelineItem, tag: string) {
    return item.linked_from.flatMap((source) => {
      const link = /^startup:([^ (]+)(?: \((.*)\))?$/.exec(source);
      if (!link || link[1].toUpperCase() !== tag.toUpperCase()) return [];
      const rawDelay = link[2] || '';
      const verified = item.verified_startup?.some(({ recipient, delay }) =>
        recipient === tag && delay === rawDelay);
      const delay = verified ? this.parseStoryDelay(rawDelay) : this.parseExactDelay(rawDelay);
      return delay === null ? [] : [delay];
    });
  }

  private autonomousStoryDay(item: TimelineItem, tag: string, completions: Record<string, number>) {
    if (item.date_country && item.date_country !== tag) return null;
    const start = this.gameDateMillis(this.startDate);
    const after = this.gameDateMillis(item.date);
    const before = this.gameDateMillis(item.date_before ?? null);
    if (start === null || (item.date && after === null) || (item.date_before && before === null)) return null;
    const focuses = (item.date_focuses ?? []).map((id) => completions[id] ??
      (this.focusById.get(id)?.completed_at_start ? 0 : Infinity));
    const day = Math.max(0, after === null ? 0 : Math.floor((after - start) / 86_400_000) + 1, ...focuses);
    return Number.isFinite(day) && (before === null || start + day * 86_400_000 < before) ? day : null;
  }

  private parseExactDelay(delay: string): number | null {
    return this.parseStoryDelay(delay, false);
  }

  private parseStoryDelay(delay: string, estimate = true): number | null {
    if (!delay.trim()) return 0;
    let hours = 0;
    for (const part of delay.split(',')) {
      const match = /^(random_)?(days|hours)\s+(\d+)$/i.exec(part.trim());
      if (!match || (match[1] && !estimate)) return null;
      hours += Number(match[3]) * (match[2].toLowerCase() === 'days' ? 24 : 1) / (match[1] ? 2 : 1);
    }
    return Math.floor(hours / 24);
  }

  private get nextLinkedStory() {
    const linked = (this.currentIsTimeline ? this.currentNextStories : this.focusSideStories)
      .filter((item) => ['event', 'news'].includes(item.kind));
    return linked.length === 1 ? linked[0] : undefined;
  }

  get continuationLabel() {
    const response = this.currentOptions[0];
    if (response) return response.text;
    if (this.nextDueStory || this.nextLinkedStory) return '继续剧情';
    if (this.focusCandidates.length) return '选择下一项国策';
    if (this.nextDevelopmentDay !== null) return '继续推进日历';
    return '查看剧情线索';
  }

  async continueRoute() {
    if (this.storyPreview || this.currentConfirmation) return;
    if (this.applyReaderDecisions()) return;
    const response = this.currentOptions[0];
    if (response) {
      if (!this.hasStoryBranches) await this.chooseStoryOption(response, false);
      return;
    }
    this.advanceAutomaticDecisions();
    this.schedulePeriodicStories();
    if (this.nextDueStory) {
      await this.readStory(this.nextDueStory);
    } else if (this.nextLinkedStory) {
      this.followStory(this.nextLinkedStory);
    } else if (!this.focusCandidates.length && this.advanceToNextDevelopment()) {
      return;
    } else {
      this.openRoutePage();
    }
  }

  private advanceToNextDevelopment() {
    const projected = this.nextProjectedFocusDay;
    const story = this.nextFutureStory;
    const storyDay = story ? this.storyScheduleDay(story) : null;
    const transfer = this.americanFactionTransfer();
    if (transfer && transfer.day > this.routeDay && transfer.day <= Math.min(projected ?? Infinity, storyDay ?? Infinity)) {
      this.saved.route_day = transfer.day;
      this.advanceAutomaticDecisions();
      this.persist();
      if (this.nextDueStory) void this.readStory(this.nextDueStory);
      else this.openRoutePage();
      return true;
    }
    if (projected !== null && (storyDay === null || projected <= storyDay)) {
      this.saved.route_day = projected;
      this.advanceAutomaticDecisions();
      this.persist();
      if (this.nextDueStory) void this.readStory(this.nextDueStory);
      else this.openRoutePage();
      return true;
    }
    if (story?.kind === 'confirmation' && storyDay !== null) {
      this.saved.route_day = Math.max(this.routeDay, storyDay);
      this.scheduleAnchoredConfirmations();
      this.persist();
      if (this.nextDueStory) void this.readStory(this.nextDueStory);
      else this.openRoutePage();
    } else if (story) this.followStory(story);
    return !!story;
  }

  get nextDevelopmentDay() {
    const story = this.nextFutureStory;
    const day = Math.min(this.nextProjectedFocusDay ?? Infinity,
      this.americanFactionTransfer()?.day ?? Infinity,
      (story ? this.storyScheduleDay(story) : null) ?? Infinity);
    return Number.isFinite(day) ? day : null;
  }

  openRoutePage() {
    this.advanceAutomaticDecisions();
    this.schedulePeriodicStories();
    const choices = this.focusCandidates;
    if (!choices.length) {
      this.setPage('chapters');
      return;
    }
    const rows = Math.ceil(choices.length / 3);
    const grouped: Chapter[] = [];
    const placed = new Set<string>();
    for (const focus of choices) {
      if (placed.has(focus.id)) continue;
      const group = [focus];
      placed.add(focus.id);
      for (let i = 0; i < group.length; i++) {
        for (const peer of choices) {
          if (placed.has(peer.id) || !this.focusesConflict(group[i], peer)) continue;
          placed.add(peer.id);
          group.push(peer);
        }
      }
      grouped.push(...group);
    }
    this.focusChoiceRows = [];
    let offset = 0;
    for (let row = 0; row < rows; row++) {
      const count = Math.ceil((choices.length - offset) / (rows - row));
      this.focusChoiceRows.push(grouped.slice(offset, offset + count).map((focus) => ({
        focus,
        image: this.imageSource(focus.image_path),
        exclusiveWith: choices.filter((peer) => peer !== focus && this.focusesConflict(focus, peer))
          .map((peer) => peer.title),
      })));
      offset += count;
    }
    this.setPage('focus-choice');
  }

  advanceToNextStory() {
    this.advanceToNextDevelopment();
  }

  restartFocusRoute() {
    this.saved.confirmations = {};
    this.saved.wartime_starts = {};
    this.saved.route_history = [];
    this.saved.option_due_days = {};
    this.saved.option_estimated_days = {};
    this.saved.country_flag_changes = undefined;
    this.saved.story_choices = {};
    this.saved.active_focus_tree = undefined;
    this.saved.decision_started = {};
    this.saved.mission_started = {};
    this.saved.mission_deadline_adjustments = {};
    this.saved.read_focus_ids = [];
    this.saved.excluded_focus_ids = [];
    this.saved.read_story_ids = [];
    this.saved.focus_completion_days = {};
    this.saved.route_day = this.openingWindowDays;
    this.saved.focus_id = '';
    this.saved.last_title = '';
    this.saved.last_kind = 'focus';
    this.persist();
    this.openRoutePage();
  }

  private sortFocuses(items: Chapter[]) {
    return [...items].sort((a, b) => {
      const pa = this.absoluteFocusPosition(a);
      const pb = this.absoluteFocusPosition(b);
      return pa.y - pb.y || pa.x - pb.x || a.title.localeCompare(b.title);
    });
  }

  private absoluteFocusPosition(
    focus: Chapter,
    path = new Set<string>(),
  ): { x: number; y: number } {
    if (path.has(focus.id)) return { x: focus.x || 0, y: focus.y || 0 };
    path.add(focus.id);
    const parent = focus.relative_position_id &&
      (this.focusVariants.get(focus.relative_position_id)?.find(
        (item) => item.focus_tree_id === focus.focus_tree_id && this.focusBelongsTo(item, this.saved.tag),
      ) ?? this.localFocus(focus.relative_position_id));
    if (!parent) return { x: focus.x || 0, y: focus.y || 0 };
    const base = this.absoluteFocusPosition(parent, path);
    return { x: base.x + (focus.x || 0), y: base.y + (focus.y || 0) };
  }

  get datedTimeline() {
    return this.countryTimeline
      .filter(
        (item) => !!item.date && !item.linked_from.length && this.storyScheduleDay(item) === null,
      )
      .sort(
        (a, b) =>
          this.dateKey(a.date).localeCompare(this.dateKey(b.date)) || a.id.localeCompare(b.id),
      );
  }

  get linkedTimeline() {
    const remaining = this.countryTimeline.filter((item) => !item.date && item.linked_from.length);
    const ordered: TimelineItem[] = [];
    const remainingKeys = new Set(remaining.map((item) => `${item.kind}:${item.id}`));
    while (remaining.length) {
      const availableIndex = remaining.findIndex((item) => {
        const parents = item.linked_from
          .map((source) => source.match(/^(event|news|decision|mission):([^/ (]+)/))
          .filter((match): match is RegExpMatchArray => !!match)
          .map((match) => `${match[1]}:${match[2]}`);
        return parents.every((parent) => !remainingKeys.has(parent));
      });
      const [item] = remaining.splice(availableIndex < 0 ? 0 : availableIndex, 1);
      remainingKeys.delete(`${item.kind}:${item.id}`);
      ordered.push(item);
    }
    return ordered;
  }

  private hasEnding(item: TimelineItem) {
    return !!(item.credits?.length || item.options.some((option) => option.credits?.length) ||
      (item.kind === 'event' && /(?:^|[\\/])TFR_events_(?:JAP|PRC)\.txt$/i.test(item.source) &&
        /^(?:jap\.1003|china\.200[0-4])$/.test(item.id)));
  }

  private shouldDeferEnding() {
    return (this.saved.tag === 'JAP' || this.saved.tag === 'PRC') && this.availableFocuses.length > 0;
  }

  private scheduleClosingCredits(source: Chapter | TimelineItem, credits: string[], day: number) {
    if (!credits.length || !this.shouldDeferEnding()) return;
    const id = `credits:${source.id}`;
    const key = `confirmation:${id}`;
    const scheduled = this.saved.confirmations ??= {};
    scheduled[key] ??= { day, item: {
      id, kind: 'confirmation', country_tag: this.saved.tag, title: '制作组致谢', description: '',
      date: null, timing: '', source: source.source, image_path: null, start_day: day, start_timing: null,
      linked_from: [], next_ids: [], options: [], credits,
    } };
  }

  private decisionAvailable(item: TimelineItem, linked = false) {
    const checks = item.decision_checks ?? [];
    const focuses = item.decision_focuses ?? [];
    if (item.kind !== 'decision' || item.never_manually_available || (item.country_tag && item.country_tag !== this.saved.tag) ||
        !this.checksPass(checks) || !focuses.every((id) =>
          this.saved.read_focus_ids.includes(id) || !!this.localFocus(id)?.completed_at_start)) return false;
    return linked || this.decisionUnlockDays(item).length > 0 || checks.length > 0 || focuses.length > 0;
  }

  private decisionUnlockDays(item: TimelineItem) {
    return item.linked_from.flatMap((source) => {
      if (!source.endsWith(' (脚本解锁)')) return [];
      const key = source.slice(0, -' (脚本解锁)'.length);
      if (key.startsWith('focus:')) {
        const id = key.slice(6);
        if (!this.saved.read_focus_ids.includes(id)) return [];
        return [this.saved.focus_completion_days[id] ?? this.routeDay];
      }
      const [story, option] = key.split('/option:');
      if (!this.saved.read_story_ids.includes(story) ||
          (option && this.saved.story_choices?.[story] !== option)) return [];
      const parent = this.timelineByKey.get(story);
      return [parent ? this.storyScheduleDay(parent) ?? this.routeDay : this.routeDay];
    });
  }

  private hasDecisionStories(item: TimelineItem) {
    return !!(item.decision_event_calls?.length || item.decision_guarded_calls?.length ||
      item.music?.length || item.decision_end_music || this.hasLocalOrBroadcastCalls(item.foreign_events) || this.hasConditionalStoryCalls(item.conditional_routes));
  }

  private hasLocalOrBroadcastCalls(links: ForeignEvent[] = []) {
    return links.some(({ recipient, target }) => recipient === this.saved.tag ||
      this.timelineById.get(target.toLowerCase())?.some((item) => item.broadcast));
  }

  private hasConditionalStoryCalls(routes: ConditionalRoutes = []): boolean {
    return routes.some((chain) => chain.some((route) => route.calls?.length ||
      this.hasLocalOrBroadcastCalls(route.foreign_events) || this.hasConditionalStoryCalls(route.nested)));
  }

  private canFollowClue(item: TimelineItem, direct: boolean) {
    if (item.hidden) return false;
    if (item.kind === 'decision') return !this.hasDecisionStories(item) && this.decisionAvailable(item, direct);
    return direct && ['event', 'news'].includes(item.kind) &&
      !this.hasEnding(item) && this.checksPass(item.trigger_checks ?? []);
  }

  get unresolvedTimeline() {
    this.refreshScheduleCache();
    return this.countryTimeline
      .filter(
        (item) =>
          !item.hidden && !item.date && ['event', 'news'].includes(item.kind) &&
          !this.saved.read_story_ids.includes(this.storyKey(item)) && this.calculateStoryDay(item, new Set()) === null,
      )
      .sort((a, b) => a.kind.localeCompare(b.kind) || a.title.localeCompare(b.title));
  }

  storyArchiveOpen = false;

  get currentIsTimeline() {
    return !!this.current && 'kind' in this.current;
  }

  get currentStoryTiming() {
    return this.current && 'kind' in this.current ? this.current.timing : '';
  }

  get currentNextStories() {
    const current = this.current;
    if (!current || !('kind' in current) || (current.options.length && !this.selectedOption))
      return [];
    return this.countryTimeline.filter((item) => {
      const source = this.storyKey(current).toLowerCase();
      if (
        item === current ||
        !this.canFollowClue(item, this.hasDirectCall(current.immediate_calls, item)) ||
        this.saved.read_story_ids.includes(this.storyKey(item)) ||
        !item.linked_from.some(
          (link) => link.toLowerCase() === source || link.toLowerCase().startsWith(source + ' ('),
        )
      ) {
        return false;
      }
      const day = this.storyScheduleDay(item);
      return day === null || day <= this.routeDay;
    });
  }

  private get storyOptions() {
    return this.current && 'kind' in this.current
      ? this.current.options.filter((option) => option.trigger_condition
          ? evaluateCondition(option.trigger_condition, (check) => this.flagMatches(check)) !== false
          : this.checksPass(option.trigger_checks ?? [])) : [];
  }

  get currentOptions() {
    return this.selectedOption ? [] : this.storyOptions;
  }

  optionTooltip(option: StoryOption | undefined): string {
    const preview = option?.effect_preview;
    if (!preview) return '';
    const labels: Record<string, string> = {
      '@conditional': '满足条件时', '@otherwise': '否则', '@random': '随机效果', '@scoped': '依游戏对象而定',
    };
    const lines = preview.lines.map(({ text, context }) =>
      context.length ? `${context.map((key) => key.startsWith('@active-mission:')
        ? this.t('任务「{0}」进行时', key.slice('@active-mission:'.length))
        : labels[key] ? this.t(labels[key]) : key).join(' · ')} — ${text}` : text,
    );
    if (preview.incomplete) lines.push(this.t('部分效果需要游戏状态才能显示。'));
    return lines.join('\n');
  }

  get showTouchEffects() {
    return this.android || this.touchEffects();
  }

  openOptionEffects(option: StoryOption | undefined, template: TemplateRef<unknown>) {
    const effects = this.optionTooltip(option);
    if (!option || !effects) return;
    this.effectTooltip = null;
    const sheet = this.bottomSheet.open(template, {
      data: { choice: option.text, effects },
      ariaLabel: this.t('查看游戏效果'),
      autoFocus: 'dialog',
      closeOnNavigation: false,
    });
    this.optionEffectsSheet = sheet;
    sheet.afterDismissed().subscribe(() => {
      if (this.optionEffectsSheet === sheet) this.optionEffectsSheet = undefined;
    });
  }

  closeOptionEffects() {
    this.optionEffectsSheet?.dismiss();
  }

  showEffectTooltip(event: MouseEvent | FocusEvent, option: StoryOption | undefined) {
    if (this.touchEffects() || ('pointerType' in event && event.pointerType === 'touch')) return;
    if (!(event instanceof MouseEvent) && (this.effectTooltip || this.android)) return;
    const text = this.optionTooltip(option);
    if (!text) return;
    const element = event.currentTarget as HTMLElement;
    const bounds = event instanceof MouseEvent ? null : element.getBoundingClientRect();
    const x = bounds ? bounds.left + bounds.width / 2 : (event as MouseEvent).clientX;
    const y = bounds ? bounds.bottom : (event as MouseEvent).clientY;
    const left = x > window.innerWidth * 0.57;
    const above = y > window.innerHeight * 0.55;
    this.effectTooltip = { text, x: x + (left ? -14 : 14), y: y + (above ? -14 : 16), left, above };
  }

  plainGameText(text: string): string {
    return this.gameText(text).map((token) => token.text).join('');
  }

  tnoBitmapRuns(text: string, font: TnoBitmapFont) {
    const runs: { text: string; glyph?: TnoBitmapGlyph }[] = [];
    let plain = '';
    for (const character of text) {
      const glyph = font[character];
      if (!glyph) {
        plain += character;
        continue;
      }
      if (plain) runs.push({ text: plain });
      plain = '';
      runs.push({ text: character, glyph });
    }
    if (plain) runs.push({ text: plain });
    return runs;
  }

  gameText(text: string) {
    const cached = this.gameTextCache.get(text);
    if (cached) return cached;
    const tokens: { text: string; color?: string; icon?: string; flag?: boolean }[] = [];
    const pattern = /§([A-Za-z0-9!])|£([A-Za-z0-9_]+)(?:\|\d+)?|(?<![\w@])@([A-Z0-9]{3})(?!\w)|\[([A-Za-z0-9_]+)\.(GetFlag|GetName)\]|\[\?[^\]\n]+\]/g;
    let cursor = 0;
    let color: string | undefined;
    const fallback: Record<string, string> = { G: '#51cc73', R: '#ff6969', Y: '#ffd45a', H: '#ffd45a', C: '#46dbff', O: '#ffa15d', W: '#fff' };
    for (const match of text.matchAll(pattern)) {
      const position = match.index ?? 0;
      if (position > cursor) tokens.push({ text: text.slice(cursor, position), color });
      if (match[1]) color = match[1] === '!' ? undefined : this.textColors[match[1]] ?? fallback[match[1]];
      else if (match[2]) {
        const name = match[2].toLowerCase();
        const sprite = this.inlineAssets[name] ?? this.inlineAssets[`gfx_${name}`];
        const tag = /^(?:gfx_)?flag_([a-z0-9]{3})$/.exec(name)?.[1]?.toUpperCase();
        if (!sprite && tag) tokens.push({ text: tag, color, icon: `flag:${tag}`, flag: true });
        else tokens.push({ text: match[2].replace(/^GFX_/i, '').replaceAll('_', ' '), color, icon: sprite });
      } else if (match[3] || match[4]) {
        const raw = (match[3] || match[4]).toUpperCase();
        const tag = raw === 'ROOT' || raw === 'THIS' ? this.saved.tag : raw;
        if (match[5] === 'GetName') tokens.push({ text: this.countryNames.get(tag) || tag || raw, color });
        else tokens.push({ text: tag || raw, color, icon: /^[A-Z0-9]{3}$/.test(tag) ? `flag:${tag}` : undefined, flag: true });
      } else tokens.push({ text: '…', color });
      cursor = position + match[0].length;
    }
    if (cursor < text.length) tokens.push({ text: text.slice(cursor), color });
    if (this.gameTextCache.size >= 256) this.gameTextCache.clear();
    this.gameTextCache.set(text, tokens);
    return tokens;
  }

  inlineImage(icon: string): string {
    if (!icon.startsWith('flag:') && /\.(png|webp|gif|jpe?g)$/i.test(icon)) return this.imageSource(icon);
    const existing = this.inlineImages.get(icon);
    if (existing !== undefined) return existing;
    const modPath = this.selectedMod?.path;
    if (!modPath) return '';
    this.inlineImages.set(icon, '');
    const request = icon.startsWith('flag:')
      ? invoke<string | null>('load_inline_flag', { modPath, gamePath: this.saved.game_path, tag: icon.slice(5) })
      : invoke<string | null>('load_super_event_image', { modPath, imagePath: icon });
    const version = this.scanVersion;
    void request.then((image) => {
      if (version !== this.scanVersion) return;
      this.inlineImages.set(icon, image ?? '');
      this.changeDetector.markForCheck();
    }).catch(() => {});
    return '';
  }

  get countryOpeningNotes() {
    return this.openingNotes[this.saved.tag] ?? [];
  }

  get hasStoryBranches() {
    const options = this.currentOptions;
    if (options.length < 2) return false;
    const signatures = new Set(
      options.map((option) =>
        JSON.stringify({
          targets: option.next_ids
            .filter(
              (id) =>
                this.optionAppliesToCountry(option, id) && this.timelineById.has(id.toLowerCase()),
            )
            .sort(),
          scene: option.super_event?.id,
          effects: [...(option.route_effects ?? [])].sort(),
          trees: option.focus_tree_changes,
          credits: option.credits,
          calls: (option.event_calls ?? []).filter(([id]) =>
            this.optionAppliesToCountry(option, id) && this.timelineById.has(id.toLowerCase()),
          ),
          foreign: (option.foreign_events ?? []).filter(({ target }) => this.timelineById.has(target.toLowerCase())),
          flags: option.flag_changes,
          missions: option.mission_activations,
          missionTime: option.mission_time_changes,
          conditional: option.conditional_routes,
          music: option.music,
          focuses: (option.focus_completions ?? []).filter(
            ({ tag }) => !tag || tag === this.saved.tag,
          ),
        }),
      ),
    );
    return signatures.size > 1;
  }

  private optionAppliesToCountry(option: StoryOption, id: string) {
    const tags = option.target_tags?.[id.toLowerCase()];
    return !tags?.length || tags.includes(this.saved.tag.toUpperCase());
  }

  private applyFocusCompletions(completions: FocusCompletion[] = [], day = this.routeDay, projected = false) {
    for (const { tag, id, bypass } of completions) {
      if ((tag && tag !== this.saved.tag) || this.saved.read_focus_ids.includes(id)) continue;
      const focus = this.localFocus(id) ?? this.routeFocuses.find((chapter) => chapter.id === id);
      if (!focus || focus.completed_at_start) continue;
      this.saved.read_focus_ids = [...this.saved.read_focus_ids, id];
      this.saved.excluded_focus_ids = [
        ...new Set([...this.saved.excluded_focus_ids, ...(focus.mutually_exclusive_ids ?? [])]),
      ];
      // A bypass satisfies prerequisites but does not run completion rewards.
      if (bypass) continue;
      this.saved.focus_completion_days = {
        ...this.saved.focus_completion_days,
        [id]: day,
      };
      this.applyFlagChanges(focus.flag_changes);
      this.playScriptedMusic(focus.music, this.saved.tag, day);
      this.applyTreeChanges(focus.focus_tree_changes);
      this.scheduleScriptCalls(focus.event_calls, day, projected);
      this.scheduleScopedCalls(focus.foreign_events, day, projected);
      this.applyFocusCompletions(focus.focus_completions, day, projected);
      this.applyConditionalRoutes(focus.conditional_routes, day, this.saved.tag, projected, focus, 'completion');
      this.activateMissions(focus.mission_activations, day);
      this.adjustMissionDeadlines(focus.mission_time_changes);
    }
  }

  private advanceFocusBypasses() {
    const day = this.routeDay;
    for (;;) {
      const completed = new Set(this.saved.read_focus_ids);
      const excluded = new Set(this.saved.excluded_focus_ids);
      const focus = this.routeFocuses.find((chapter) => chapter.bypass_condition &&
        !chapter.completed_at_start && !completed.has(chapter.id) && !excluded.has(chapter.id) &&
        !chapter.mutually_exclusive_ids?.some((id) => completed.has(id)) &&
        (chapter.prerequisite_groups ?? []).every((group) => group.some((id) => completed.has(id) || this.localFocus(id)?.completed_at_start)) &&
        (!chapter.branch_condition || evaluateCondition(chapter.branch_condition, (check) => this.flagMatches(check, day)) === true) &&
        evaluateCondition(chapter.bypass_condition, (check) => this.flagMatches(check, day)) === true);
      if (!focus) break;
      const count = this.saved.read_focus_ids.length;
      this.applyFocusCompletions([{ id: focus.id, tag: null, bypass: true }], day);
      if (this.saved.read_focus_ids.length === count) break;
      this.persist();
    }
  }

  private applyProjectedFocusCompletions() {
    const tag = this.saved.tag;
    for (const entry of this.projectedForeignCompletions.get(tag) ?? []) {
      if (entry.day > this.routeDay || entry.origin === tag) continue;
      if (this.saved.read_focus_ids.includes(entry.id)) continue;
      this.applyFocusCompletions([entry], entry.day, entry.projected);
      this.persist();
    }
  }

  private get nextProjectedFocusDay() {
    const tag = this.saved.tag;
    const entries = this.projectedForeignCompletions.get(tag) ?? [];
    return entries.find((entry) => entry.origin !== tag && entry.day > this.routeDay &&
      !this.saved.read_focus_ids.includes(entry.id) && this.localFocus(entry.id))?.day ?? null;
  }

  get readerDate() {
    const item = this.current;
    if (!item) return null;
    if ('kind' in item) {
      const day = this.currentScheduledDay;
      return day === null ? this.formatGameDate(item.date) : this.calendarLabel(day);
    }
    const day = this.saved.focus_completion_days[item.id];
    return day === undefined ? null : this.calendarLabel(day);
  }

  get readerLanguage() {
    return { english: 'en', simp_chinese: 'zh-Hans', traditional_chinese: 'zh-Hant', japanese: 'ja' }[
      this.saved.language
    ];
  }

  private get readerRelatedStories() {
    return [...new Set([...this.optionDestinations, ...this.focusSideStories, ...this.currentNextStories])];
  }

  get readerLinks() {
    const automatic = this.nextLinkedStory;
    return this.readerRelatedStories.filter((story) => story !== automatic && story.kind !== 'decision');
  }

  get readerDecisions() {
    return this.readerRelatedStories.filter((story) => story.kind === 'decision');
  }

  isDecisionChecked(id: string) {
    return this.checkedDecisions.has(id);
  }

  toggleReaderDecision(id: string, checked: boolean) {
    if (checked) this.checkedDecisions.add(id);
    else this.checkedDecisions.delete(id);
  }

  private applyReaderDecisions() {
    if (!this.checkedDecisions.size) return;
    const selected = this.readerDecisions.filter(({ id }) => this.checkedDecisions.has(id));
    this.checkedDecisions.clear();
    if (!selected.length) return;
    this.checkpointRoute('event');
    const scenes = selected.flatMap((item) => {
      const scene = this.activateDecision(item);
      return scene ? [scene] : [];
    });
    this.persist();
    if (!scenes.length) return false;
    const play = (index: number) => {
      if (index >= scenes.length) void this.continueRoute();
      else this.openSuperEvent(scenes[index], () => play(index + 1));
    };
    play(0);
    return true;
  }

  private storyText(variants: TextVariant[] = []) {
    return variants.find((variant) =>
      variant.completed_focuses.every((id) => this.saved.read_focus_ids.includes(id)) &&
      this.checksPass(variant.flags),
    )?.text;
  }

  get currentStoryTitle() {
    if (this.searchPreview && this.searchMatch?.kind === 'title') return this.searchMatch.raw;
    const item = this.current;
    if (item && 'kind' in item && item.mission?.shared_resolution) {
      return item.id === 'second_nato_war_mission' ? '第二次欧洲战争的结局' : '第三次欧洲战争的结局';
    }
    if (this.currentForeignMission) return this.currentForeignMissionTitle;
    return (item && 'kind' in item ? this.storyText(item.title_variants) : undefined) ?? item?.title;
  }

  get currentStoryDescription() {
    if (this.searchPreview && this.searchMatch?.kind === 'description') return this.searchMatch.raw;
    const item = this.current;
    if (item && 'kind' in item && item.mission?.shared_resolution) return '请选择这场战事的结果，阅读随后发生的新闻与事件。';
    if (this.currentForeignMission) return '请选择这场战事的结果，阅读随后发生的新闻与事件。';
    return (item && 'kind' in item ? this.storyText(item.description_variants) : undefined) ??
      item?.description;
  }

  get currentCredits() {
    if (this.shouldDeferEnding()) return [];
    const options = this.storyOptions;
    const common = options[0]?.credits?.filter(
      (credit) => options.every((option) => option.credits?.includes(credit)),
    ) ?? [];
    const declared = this.current && 'kind' in this.current ? this.current.options : [];
    const creditsWithoutMatchingOption = !options.length && declared.length && declared.every((option) => option.credits?.length)
      ? declared[0].credits : [];
    return [...new Set([
      ...(this.current?.credits ?? []),
      ...(this.selectedOption?.credits ?? common),
      ...(creditsWithoutMatchingOption || []),
      ...(this.current && 'kind' in this.current ? this.japaneseDefeatCredits(this.current) : []),
    ])];
  }

  get optionDestinations() {
    if (!this.selectedOption || !this.current || !('kind' in this.current)) return [];
    const option = this.selectedOption;
    return this.optionDestinationsFor(option).filter((item) =>
      this.canFollowClue(item, this.hasDirectCall(option.event_calls, item)) &&
      item !== this.current &&
      !this.saved.read_story_ids.includes(this.storyKey(item)) &&
      this.saved.option_due_days?.[this.storyKey(item)] === undefined,
    );
  }

  private optionDestinationsFor(option: StoryOption) {
    if (!this.current || !('kind' in this.current)) return [];
    const relation =
      `${this.current.kind}:${this.current.id}/option:${option.id}`.toLocaleLowerCase();
    const ids = new Set(
      option.next_ids
        .filter((id) => this.optionAppliesToCountry(option, id))
        .map((id) => id.toLocaleLowerCase()),
    );
    return this.countryTimeline.filter(
      (item) =>
        ids.has(item.id.toLocaleLowerCase()) &&
        (item.broadcast || this.focusEventApplies(option, item, this.saved.tag)) &&
        item.linked_from.some((source) => source.toLocaleLowerCase().startsWith(relation)),
    );
  }

  get lastStory() {
    return this.timeline.find(
      (item) => item.id === this.saved.focus_id && item.kind === this.saved.last_kind,
    );
  }

  dateKey(date: string | null) {
    return (date || '')
      .split('.')
      .map((part) => part.padStart(2, '0'))
      .join('.');
  }

  kindLabel(kind: string) {
    return (
      (
        {
          event: '国家事件',
          focus: '国策',
          news: '国际新闻',
          decision: '决议',
          mission: '限时任务',
          'opening-note': '开局提示',
        } as Record<string, string>
      )[kind] || '剧情'
    );
  }

  get lastChapter() {
    return (
      this.chapters.find((chapter) => chapter.id === this.saved.focus_id) ||
      (this.saved.focus_id
        ? {
            id: this.saved.focus_id,
            title: this.saved.last_title || this.saved.focus_id,
            description: '',
            icon: '',
            image_available: false,
            image_path: null,
            source: '',
          }
        : undefined)
    );
  }

  imageSource(path: string | null | undefined) {
    return path ? convertFileSrc(path) : '';
  }

  async ngOnInit() {
    try {
      this.saved = {
        ...emptySaved,
        ...JSON.parse(localStorage.getItem('hoi4reader') || '{}'),
      } as Saved;
      if (this.saved.music_transition !== 'queue') this.saved.music_transition = 'interrupt';
      if (!['auto', 'material', 'tfr', 'tno'].includes(this.saved.theme) ||
          (!this.saved.theme_preference_set && this.saved.theme === 'material')) this.saved.theme = 'auto';
      if (!['material', 'tfr', 'tno'].includes(this.saved.last_mod_theme ?? '')) this.saved.last_mod_theme = undefined;
      if (!this.langOptions.some(({ value }) => value === this.saved.language)) this.saved.language = 'english';
      document.documentElement.lang = this.readerLanguage;
      document.documentElement.dataset['readerTheme'] = this.theme;
      this.saved.read_focus_ids ||= [];
      this.saved.excluded_focus_ids ||= [];
      this.saved.read_story_ids ||= [];
      this.saved.focus_completion_days ||= {};
      const discovery = await invoke<Discovery>('discover');
      this.saved.game_path ||= discovery.game_path;
      this.saved.workshop_path ||= discovery.workshop_path;
      this.mods = discovery.mods || [];
      if (this.saved.mod_path) {
        this.selectedMod = this.mods.find((mod) => mod.path === this.saved.mod_path) || {
          name: this.saved.last_mod || this.saved.mod_path.split(/[\\/]/).pop() || 'Mod',
          path: this.saved.mod_path,
          workshop_id: '',
          image_path: null,
        };
        this.selectedCountry = {
          tag: this.saved.tag,
          name: this.saved.last_country || this.saved.tag,
          chapter_count: 0,
          story_count: 0,
        };
      }
      void this.loadTfrMenuArt();
    } catch (error) {
      this.error = String(error);
    } finally {
      this.changeDetector.markForCheck();
    }
  }

  private saveTimer?: ReturnType<typeof setTimeout>;

  persist() {
    this.saveTimer ??= setTimeout(() => this.flushProgress(), 75);
  }

  @HostListener('window:pagehide')
  flushProgress() {
    if (this.saveTimer === undefined) return;
    clearTimeout(this.saveTimer);
    this.saveTimer = undefined;
    localStorage.setItem('hoi4reader', JSON.stringify(this.saved));
  }

  async refreshMods() {
    this.loading = true;
    try {
      const discovery = await invoke<Discovery>('discover');
      this.saved.game_path = discovery.game_path || this.saved.game_path;
      this.saved.workshop_path = discovery.workshop_path || this.saved.workshop_path;
      this.mods = this.saved.workshop_path
        ? await invoke<ModInfo[]>('scan_workshop_dir', {
            root: this.saved.workshop_path,
          })
        : discovery.mods || [];
      void this.loadTfrMenuArt();
      this.persist();
    } catch (error) {
      this.error = String(error);
    } finally {
      this.loading = false;
      this.changeDetector.markForCheck();
    }
  }

  scan(force = false): Promise<boolean> {
    if (!this.selectedMod) return Promise.resolve(false);
    const { path } = this.selectedMod;
    const { language } = this.saved;
    const key = JSON.stringify([path, language, this.saved.game_path]);
    if (this.activeScan?.key === key) return this.activeScan.result;
    if (!force && this.loadedScanKey === key) return Promise.resolve(true);
    if (this.loadedScanKey !== key) {
      this.countries = [];
      this.hasSecondaryCountries = false;
    }
    this.allIndexedCountries = null;
    this.secondaryIndexPending = false;
    this.cacheWarning = '';
    this.loadedScanKey = null;
    this.countryPreview = [];
    this.pendingCountryTag = null;
    this.loading = true;
    this.error = '';
    const version = ++this.scanVersion;
    const result = Promise.resolve().then(() => this.runScan(path, language, key, version, force));
    this.activeScan = { key, result };
    this.changeDetector.markForCheck();
    return result;
  }

  private async runScan(path: string, language: Lang, key: string, version: number, force: boolean): Promise<boolean> {
    if (version !== this.scanVersion) return false;
    try {
      const onCountries = new Channel<Pick<Country, 'tag' | 'name' | 'major_order'>[]>((countries) => {
        if (version !== this.scanVersion || !this.activeScan) return;
        this.countryPreview = countries.map((country) => ({ ...country, chapter_count: 0, story_count: 0 }));
        this.changeDetector.markForCheck();
      });
      const result = await invoke<ScanResult>('scan_mod', {
        path,
        language,
        gamePath: this.saved.game_path,
        force,
        onCountries,
      });
      if (version !== this.scanVersion) return false;
      this.startDate = result.start_date;
      this.countries = result.countries;
      this.inlineAssets = result.inline_assets ?? {};
      this.textColors = result.text_colors ?? {};
      this.inlineImages.clear();
      this.gameTextCache.clear();
      this.openingNotes = result.opening_notes ?? {};
      this.countryNames = new Map((result.country_names ?? result.countries).map(({ tag, name }) => [tag, name]));
      this.hasSecondaryCountries = result.has_secondary_countries ?? false;
      this.cacheWarning = result.cache_warning ?? '';
      this.chapters = result.chapters;
      this.timeline = result.timeline || [];
      this.searchDocuments = null;
      if (this.page === 'search') this.updateSearch();
      this.initialCountryFlags = result.initial_country_flags ?? {};
      this.defaultGameRules = result.default_game_rules ?? {};
      this.greatAsianWarStart = result.great_asian_war_start ?? null;
      this.greatAsianWarRecipients = result.great_asian_war_recipients ?? [];
      this.greatAsianWarNews = result.great_asian_war_news ?? null;
      this.wartimeHooks = result.wartime_hooks ?? [];
      this.americanCrisisAnchor = result.american_crisis_anchor ?? null;
      this.warNewsMissions = result.war_news_missions ?? {};
      this.indexTimeline();
      this.restoreLegacyElectionChoice();
      this.restoreFocusTree();
      this.restoreMissions();
      this.diagnostics = result.diagnostics;
      if (
        !this.saved.route_day ||
        (!Object.keys(this.saved.focus_completion_days).length &&
          this.saved.last_kind === 'focus' && this.saved.read_focus_ids.includes(this.saved.focus_id))
      ) {
        let day = this.openingWindowDays;
        for (const id of this.saved.read_focus_ids) {
          const focus = this.chapters.find((chapter) => chapter.id === id);
          if (!focus) continue;
          day += focus.duration_days || 70;
          this.saved.focus_completion_days[id] = day;
        }
        this.saved.route_day = day;
        this.persist();
      }
      this.restoreTaiwanLoss();
      this.restoreEnvironmentalOutcome();
      this.loadedScanKey = key;
      return true;
    } catch (error) {
      if (version === this.scanVersion) this.error = String(error);
      return false;
    } finally {
      if (version === this.scanVersion) {
        const pending = this.countries.find((country) => country.tag === this.pendingCountryTag);
        this.activeScan = null;
        this.loading = false;
        this.countryPreview = [];
        this.pendingCountryTag = null;
        if (this.loadedScanKey === key && this.page === 'countries') {
          if (pending) this.chooseCountry(pending);
          else if (this.showAllCountries) void this.loadSecondaryCountries();
        }
        this.changeDetector.markForCheck();
      }
    }
  }

  setShowAllCountries(enabled: boolean) {
    this.showAllCountries = enabled;
    if (enabled) void this.loadSecondaryCountries();
  }

  private async loadSecondaryCountries() {
    if (!this.hasSecondaryCountries || !this.loadedScanKey || !this.selectedMod ||
      this.allIndexedCountries || this.secondaryIndexPending) return;
    const version = this.scanVersion;
    this.secondaryIndexPending = true;
    this.changeDetector.markForCheck();
    try {
      const result = await invoke<{ countries: Country[]; cache_warning: string | null }>('index_secondary_countries', {
        path: this.selectedMod.path,
        language: this.saved.language,
      });
      if (version !== this.scanVersion) return;
      this.allIndexedCountries = [...this.countries, ...result.countries];
      if (result.cache_warning) this.cacheWarning = result.cache_warning;
    } catch (error) {
      if (version === this.scanVersion) {
        this.showAllCountries = false;
        this.error = String(error);
      }
    } finally {
      if (version === this.scanVersion) {
        this.secondaryIndexPending = false;
        this.changeDetector.markForCheck();
      }
    }
  }

  private restoreLegacyElectionChoice() {
    if (this.saved.tag !== 'JAP' || this.saved.country_flag_changes !== undefined) return;
    const election = this.timelineByKey.get('event:jap.16');
    if (!election || !/(?:^|[\\/])TFR_events_JAP\.txt$/i.test(election.source)) return;
    const route = this.saved;
    const evidence = (id: string) =>
      route.read_story_ids.includes(`event:${id}`) ||
      route.option_due_days?.[`event:${id}`] !== undefined;
    const ldp = evidence('jap.114');
    const opposition = evidence('jap.113');
    if (ldp === opposition) return;
    const option = election.options.find(({ next_ids }) => next_ids.includes(ldp ? 'jap.114' : 'jap.113'));
    if (!option) return;
    this.applyFlagChanges(election.flag_changes);
    this.applyFlagChanges(option.flag_changes);
    route.story_choices = { ...route.story_choices, 'event:jap.16': option.id };
    this.persist();
  }

  private restoreFocusTree() {
    if (this.saved.active_focus_tree !== undefined) return;
    for (const key of this.saved.read_story_ids) {
      const story = this.timelineByKey.get(key);
      if (!story) continue;
      const chosen = story.options.find(({ id }) => id === this.saved.story_choices?.[key]);
      const trees = [...(story.focus_tree_changes ?? []), ...(chosen?.focus_tree_changes ?? [])];
      if (!trees.length) continue;
      this.applyTreeChanges(trees);
      if (!this.saved.active_focus_tree || !trees.includes(this.saved.active_focus_tree)) continue;
      this.applyFocusCompletions(
        [...(story.focus_completions ?? []), ...(chosen?.focus_completions ?? [])],
        this.storyScheduleDay(story) ?? this.routeDay,
      );
    }
    if (this.saved.active_focus_tree !== undefined) this.persist();
  }

  private indexTimeline() {
    this.timelineById.clear();
    this.timelineByKey.clear();
    this.periodicStoriesByOwner.clear();
    this.autonomousStoriesByOwner.clear();
    this.manualTriggersByOwner.clear();
    this.focusProgressStories.clear();
    this.projectedForeignCompletions.clear();
    for (const item of this.timeline) {
      const manualOwner = item.manual_trigger?.owner;
      if (manualOwner) {
        const stories = this.manualTriggersByOwner.get(manualOwner) ?? [];
        stories.push(item);
        this.manualTriggersByOwner.set(manualOwner, stories);
      }
      const focus = item.focus_progress_trigger?.focus;
      if (focus) {
        const stories = this.focusProgressStories.get(focus) ?? [];
        stories.push(item);
        this.focusProgressStories.set(focus, stories);
      }
      const owner = item.autonomous_trigger?.owner;
      if (owner && !item.hidden) {
        const stories = this.autonomousStoriesByOwner.get(owner) ?? [];
        stories.push(item);
        this.autonomousStoriesByOwner.set(owner, stories);
      }
      for (const owner of new Set(item.periodic_triggers?.map((trigger) => trigger.owner))) {
        const stories = this.periodicStoriesByOwner.get(owner) ?? [];
        stories.push(item);
        this.periodicStoriesByOwner.set(owner, stories);
      }
      this.timelineByKey.set(`${item.kind}:${item.id}`, item);
      const idKey = item.id.toLocaleLowerCase();
      const sameId = this.timelineById.get(idKey) || [];
      sameId.push(item);
      this.timelineById.set(idKey, sameId);
    }
    this.focusById.clear();
    this.focusVariants.clear();
    this.focusConditionFlags = new Set();
    const remember = (condition?: ScriptCondition<CountryFlag>) => {
      switch (condition?.op) {
        case 'check': this.focusConditionFlags.add(condition.args.flag); break;
        case 'all': case 'any': condition.args.forEach(remember); break;
        case 'not': remember(condition.args); break;
      }
    };
    for (const focus of this.chapters) {
      remember(focus.branch_condition);
      remember(focus.availability_condition);
      const existing = this.focusById.get(focus.id);
      if (!existing || (focus.initial_tree && !existing.initial_tree)) this.focusById.set(focus.id, focus);
      const variants = this.focusVariants.get(focus.id) ?? [];
      variants.push(focus);
      this.focusVariants.set(focus.id, variants);
    }
    this.indexExternalDevelopments();
    this.indexForeignSchedule();
    this.indexProjectedFocuses();
    this.countryTimelineCacheTag = null;
    this.scheduleCache.clear();
    this.scheduledStoryCache = undefined;
    this.scheduleCacheFocusSignature = '';
  }

  private focusProgressStories = new Map<string, TimelineItem[]>();
  private projectedFocusDays = new Map<string, number>();
  private projectedForeignCompletions = new Map<string, (FocusCompletion & { day: number; origin: string; projected: boolean })[]>();

  private indexForeignCompletion(origin: string, completion: FocusCompletion, day: number, projected: boolean) {
    if (!completion.tag || completion.tag === origin) return;
    const entries = this.projectedForeignCompletions.get(completion.tag) ?? [];
    if (entries.some((entry) => entry.id === completion.id && entry.origin === origin &&
      entry.day === day && entry.bypass === completion.bypass && entry.projected === projected)) return;
    entries.push({ ...completion, origin, day, projected });
    this.projectedForeignCompletions.set(completion.tag, entries);
  }

  private focusProgressDay(item: TimelineItem, start: number, end: number) {
    const trigger = item.focus_progress_trigger;
    if (!trigger || end <= start) return null;
    const elapsed = (end - start) * trigger.progress;
    return start + (trigger.inclusive ? Math.ceil(elapsed) : Math.floor(elapsed) + 1);
  }

  private automaticForeignDay(item: TimelineItem): number | null {
    const tag = this.saved.tag.toUpperCase();
    const key = this.storyKey(item);
    const scoped = this.foreignStoryDays.get(`${tag}:${key}`);
    const world = [...(this.worldNewsDays.get(key) ?? [])]
      .filter(([owner]) => owner !== tag)
      .map(([, day]) => day);
    const day = Math.min(scoped ?? Infinity, ...world);
    return Number.isFinite(day) ? day : null;
  }

  private calendarDay(date: string) {
    const start = this.gameDateMillis(this.startDate);
    const target = this.gameDateMillis(date);
    return start === null || target === null ? null : Math.max(0, Math.round((target - start) / 86_400_000));
  }

  private projectedForeignDay(item: TimelineItem): number | null {
    const tag = this.saved.tag;
    if (tag !== 'USA' && item.kind === 'event' && item.id === this.americanCrisisAnchor) {
      // Community timeline places the outbreak in late 2021 / early 2022.
      // This is the reader's chosen crisis date; the script supplies the following 21-day delay.
      return this.calendarDay('2021.12.11');
    }
    if (tag !== 'SOV' && tag !== 'UKR' && item.kind === 'news' && item.id === 'news.237') {
      return this.europeanWarDay;
    }
    const start = this.greatAsianWarStart;
    const isStart = item.kind === 'event' && item.id === start?.target && tag === start.recipient;
    const isNews = tag !== 'PRC' && item.kind === 'news' && item.id === this.greatAsianWarNews?.[0];
    if (isStart || isNews) {
      const target = this.calendarDay('2028.1.1');
      if (target === null) return null;
      let base = target;
      if (tag === 'JAP') {
        const outcome = ['event:jap.9', 'event:jap.10'].find((key) => this.saved.read_story_ids.includes(key));
        if (!outcome) return null;
        base = Math.max(base, (this.saved.option_due_days?.[outcome] ?? 0) + 365);
      }
      const delay = this.parseExactDelay((isStart ? start?.delay : this.greatAsianWarNews?.[1]) ?? '');
      return delay === null ? null : base + delay;
    }
    const key = this.storyKey(item);
    const dates = [...(this.projectedNewsDays.get(key) ?? [])]
      .filter(([owner]) => owner !== tag).map(([, day]) => day);
    const day = Math.min(this.projectedStoryDays.get(`${tag}:${key}`) ?? Infinity, ...dates);
    return Number.isFinite(day) ? day : null;
  }

  isProjectedStory(item: TimelineItem, visited = new Set<string>()): boolean {
    const key = this.storyKey(item);
    if (visited.has(key)) return false;
    const day = this.storyScheduleDay(item);
    if (day === null) return false;
    if (item.kind === 'confirmation') return !!item.confirmation?.projected;
    if (item.kind === 'mission' && (this.foreignMissionStart(item) !== null || item.mission?.shared_resolution)) return true;
    if (this.automaticForeignDay(item) === day) return false;
    if (this.saved.option_due_days?.[key] === day) return this.saved.option_estimated_days?.[key] === day;
    if (item.date_after) return false;
    const opening = item.linked_from.flatMap((source) => {
      const link = /^startup:([^ (]+)(?: \((.*)\))?$/.exec(source);
      return link?.[1] === this.saved.tag && this.parseStoryDelay(link[2] || '') === day ? [link[2] || ''] : [];
    });
    if (opening.length) return opening.every((delay) => this.parseExactDelay(delay) === null);
    if (this.projectedForeignDay(item) === day) return true;
    visited.add(key);
    return item.linked_from.some((source) => {
      const match = /^(event|news|focus|decision):([^/ (]+)(?: \((.*)\))?$/.exec(source);
      if (!match) return false;
      const rawDelay = match[3] || '';
      const delay = this.parseStoryDelay(rawDelay);
      if (delay === null) return false;
      const estimated = this.parseExactDelay(rawDelay) === null;
      if (match[1] === 'focus') {
        const focus = this.localFocus(match[2]) ?? this.focusById.get(match[2]);
        return estimated && this.focusEventApplies(focus, item, this.saved.tag) &&
          (this.hasDirectCall(focus?.event_calls, item, rawDelay) || !!focus?.foreign_events?.some(
            (link) => link.recipient === this.saved.tag && link.target.toLowerCase() === item.id.toLowerCase() &&
              link.delay === rawDelay,
          )) && (this.saved.focus_completion_days[match[2]] ?? Infinity) + delay === day;
      }
      const parent = this.timelineByKey.get(`${match[1]}:${match[2]}`);
      if (!parent) return false;
      if (parent.kind === 'decision') {
        return estimated && (this.saved.decision_started?.[parent.id] ?? Infinity) + delay === day &&
          !!parent.decision_guarded_calls?.some((call) => call.deferred &&
            call.target.toLowerCase() === item.id.toLowerCase() && call.delay === rawDelay && this.checksPass(call.checks));
      }
      const scoped = parent.foreign_events?.some((link) =>
        link.target.toLowerCase() === item.id.toLowerCase() && link.recipient === this.saved.tag &&
        link.delay === (match[3] || ''),
      );
      if (!scoped && !this.isImmediateParent(parent, item)) return false;
      const starts = this.startupDays(parent, parent.country_tag);
      const parentDay = starts.length ? Math.min(...starts) : this.storyScheduleDay(parent);
      return parentDay !== null && parentDay + delay === day && (estimated || this.isProjectedStory(parent, visited));
    });
  }

  get readerDateIsProjected() {
    return !!this.current && 'kind' in this.current && this.isProjectedStory(this.current);
  }

  private indexProjectedFocuses() {
    this.projectedStoryDays.clear();
    this.projectedNewsDays.clear();
    this.projectedFocusDays.clear();
    this.europeanWarDay = null;
    this.indexProjectedEuropeanIntervention();
    const gawDay = this.calendarDay('2028.1.1');
    if (gawDay !== null) {
      this.indexProjectedFocusEffects('PRC', gawDay,
        this.greatAsianWarRecipients.filter(({ recipient }) => recipient !== 'JAP'));
    }
    for (const { owner, focus, start, day, selection, rewards, foreignCompletions, bypassed } of projectForeignFocuses(
      this.chapters, this.initialCountryFlags, this.defaultGameRules,
    )) {
      const key = `${owner}:${focus.id}`;
      this.projectedFocusDays.set(key, Math.min(day, this.projectedFocusDays.get(key) ?? Infinity));
      if (start !== null) {
        this.indexProjectedFocusEffects(owner, start, focus.selection_foreign_events, focus.selection_broadcasts, focus.selection_calls);
        for (const route of selection) this.indexProjectedFocusEffects(owner, start, route.foreign_events, [], route.calls);
        for (const event of this.focusProgressStories.get(focus.id) ?? []) {
          const due = this.focusProgressDay(event, start, day);
          if (due !== null) this.indexProjectedFocusEffects(owner, due, [], [], [[event.id, '']]);
        }
      }
      if (!bypassed) this.indexProjectedFocusEffects(owner, day, focus.foreign_events, focus.broadcast_calls, focus.event_calls);
      for (const route of rewards) this.indexProjectedFocusEffects(owner, day, route.foreign_events, [], route.calls);
      for (const completion of foreignCompletions) {
        this.indexForeignCompletion(owner, completion, completion.day, true);
      }
    }
    this.indexProjectedAmericanOutbreak();
    for (const entries of this.projectedForeignCompletions.values()) entries.sort((a, b) => a.day - b.day);
  }

  private indexProjectedFocusEffects(
    owner: string, day: number, foreign: ForeignEvent[] = [], broadcasts: [string, string][] = [],
    local: [string, string][] = [], excludedReader = '',
  ) {
    const queue: { owner: string; item: TimelineItem; day: number }[] = [];
    const seen = new Map<string, number>();
    const enqueue = (recipient: string, target: string, rawDelay: string, start: number, newsOnly = false) => {
      const delay = this.parseStoryDelay(rawDelay);
      if (delay === null) return;
      for (const item of this.timelineById.get(target.toLowerCase()) ?? []) {
        if (!['event', 'news'].includes(item.kind) || (newsOnly && !item.broadcast) ||
            (item.startup_trigger !== '' && item.startup_trigger !== recipient)) continue;
        const key = this.storyKey(item);
        const due = start + delay;
        const scopedKey = `${recipient}:${key}`;
        if (due >= (seen.get(scopedKey) ?? Infinity)) continue;
        seen.set(scopedKey, due);
        if (item.broadcast) {
          const dates = this.projectedNewsDays.get(key) ?? new Map<string, number>();
          const source = excludedReader || owner;
          dates.set(source, Math.min(due, dates.get(source) ?? Infinity));
          this.projectedNewsDays.set(key, dates);
        } else if (recipient !== owner) {
          this.projectedStoryDays.set(scopedKey, Math.min(due, this.projectedStoryDays.get(scopedKey) ?? Infinity));
        }
        queue.push({ owner: recipient, item, day: due });
      }
    };
    for (const link of foreign) enqueue(link.recipient, link.target, link.delay, day);
    for (const [target, delay] of broadcasts) enqueue(owner, target, delay, day, true);
    for (const [target, delay] of local) enqueue(owner, target, delay, day);
    for (let i = 0; i < queue.length; i++) {
      const step = queue[i];
      const effects = this.automaticEventCalls(step.item);
      if (!step.item.broadcast) {
        for (const completion of effects.completions) this.indexForeignCompletion(step.owner, completion, step.day, true);
        for (const { owner: tag, focus } of this.completedFocusChain(step.owner, effects.completions)) {
          for (const [target, delay] of focus.event_calls ?? []) enqueue(tag, target, delay, step.day);
          for (const link of focus.foreign_events ?? []) enqueue(link.recipient, link.target, link.delay, step.day);
        }
      }
      for (const link of effects.foreign) {
        enqueue(link.recipient, link.target, link.delay, step.day);
      }
      for (const [target, delay] of effects.local) {
        // A broadcast may be read outside its original country. Do not
        // deliver its local effects to every country that reads the bulletin.
        enqueue(step.owner, target, delay, step.day, !!step.item.broadcast);
      }
    }
  }

  private automaticEventCalls(item: TimelineItem) {
    // Advance only effects shared by all unconditional responses. The
    // selected country's own event options are still evaluated when read.
    const options = item.options;
    const deterministic = options.length && options.every((option) =>
      !option.trigger_checks?.length && !option.trigger_condition);
    const first = options[0];
    const commonCalls = deterministic ? (first.event_calls ?? []).filter(([id, delay]) =>
      options.every((option) => option.event_calls?.some(([target, timing]) => target === id && timing === delay))) : [];
    const commonForeign = deterministic ? (first.foreign_events ?? []).filter((link) =>
      options.every((option) => option.foreign_events?.some(({ recipient, target, delay }) =>
        recipient === link.recipient && target === link.target && delay === link.delay))) : [];
    const commonCompletions = deterministic ? (first.focus_completions ?? []).filter((focus) =>
      options.every((option) => option.focus_completions?.some(({ id, tag, bypass }) =>
        id === focus.id && tag === focus.tag && bypass === focus.bypass))) : [];
    return {
      local: [...(item.immediate_calls ?? []), ...commonCalls],
      foreign: [...(item.foreign_events ?? []), ...commonForeign],
      completions: [...(item.focus_completions ?? []), ...commonCompletions],
    };
  }

  private indexProjectedAmericanOutbreak() {
    if (!this.americanCrisisAnchor) return;
    const emergency = this.timelineByKey.get(`event:${this.americanCrisisAnchor}`);
    const outbreak = this.timelineByKey.get('event:usa.104');
    const call = emergency?.immediate_calls?.find(([id]) => id === outbreak?.id);
    const start = this.calendarDay('2021.12.11');
    const delay = call ? this.parseExactDelay(call[1]) : null;
    if (!emergency || !outbreak || start === null || delay === null) return;
    const transfer = this.timelineByKey.get('event:usa.15');
    const split = this.timelineByKey.get('event:usa.6');
    const emergencyLink = transfer?.foreign_events?.find(({ recipient, target }) =>
      recipient === 'USC' && target === emergency.id);
    const transferLink = split?.immediate_calls?.find(([id]) => id === transfer?.id);
    const emergencyDelay = emergencyLink ? this.parseExactDelay(emergencyLink.delay) : null;
    const transferDelay = transferLink ? this.parseExactDelay(transferLink[1]) : null;
    if (split && transfer && emergencyDelay !== null && transferDelay !== null) {
      this.indexProjectedFocusEffects('USA', start - emergencyDelay - transferDelay,
        split.foreign_events, split.broadcast_calls, split.immediate_calls);
    }
    this.indexProjectedFocusEffects('USC', start, emergency.foreign_events,
      emergency.broadcast_calls, [], 'USA');
    this.indexProjectedFocusEffects('USC', start + delay, outbreak.foreign_events,
      outbreak.broadcast_calls, [], 'USA');
  }

  private indexProjectedEuropeanIntervention() {
    const missionId = 'NATO_intervention_timer';
    const buildup = this.timelineByKey.get('event:nato.15');
    const mission = this.timelineByKey.get(`mission:${missionId}`)?.mission;
    // The buildup date is a reader projection. The intervention and its
    // follow-ups keep the delays of the installed mod's mission.
    const buildupDay = this.calendarDay('2024.11.2');
    if (!buildup?.mission_activations?.includes(missionId) || !mission || buildupDay === null) return;
    const newsCall = mission.timeout.calls.find(([id]) => id === 'news.237');
    const newsDelay = newsCall ? this.parseExactDelay(newsCall[1]) : null;
    if (newsDelay === null) return;
    const interventionDay = buildupDay + mission.days;
    this.europeanWarDay = interventionDay + newsDelay;
    this.indexProjectedFocusEffects('SOV', interventionDay, mission.timeout.foreign_events.filter(
      ({ recipient }) => recipient !== 'UKR' && recipient !== 'SOV',
    ));
  }

  private indexForeignSchedule() {
    this.foreignStoryDays.clear();
    this.worldNewsDays.clear();
    const earliest = new Map<string, number>();
    const queue: { item: TimelineItem; tag: string; day: number }[] = [];
    const enqueue = (item: TimelineItem, tag: string, day: number) => {
      const key = `${tag}:${this.storyKey(item)}`;
      if (day >= (earliest.get(key) ?? Infinity)) return;
      earliest.set(key, day);
      if (item.broadcast) {
        const story = this.storyKey(item);
        const dates = this.worldNewsDays.get(story) ?? new Map<string, number>();
        dates.set(tag, Math.min(day, dates.get(tag) ?? Infinity));
        this.worldNewsDays.set(story, dates);
      }
      queue.push({ item, tag, day });
    };
    const allowed = (item: TimelineItem, tag: string) =>
      item.startup_trigger === '' || item.startup_trigger === tag;
    for (const item of this.timeline) {
      if (item.date_after && !item.date_focuses?.length) {
        const owner = item.date_country || item.country_tag || 'ZZZ';
        const day = this.autonomousStoryDay(item, owner, {});
        if (day !== null) enqueue(item, owner, day);
      }
      for (const { recipient, delay } of item.verified_startup ?? []) {
        const day = this.parseExactDelay(delay);
        if (day === null) continue;
        const taiwanFallback = recipient === 'PRC' && item.id === 'china.124' &&
          /(?:^|[\\/])TFR_events_PRC\.txt$/i.test(item.source);
        if (allowed(item, recipient) || taiwanFallback) enqueue(item, recipient, day);
      }
    }
    const routeEffects = (
      owner: string, day: number, calls: [string, string][] = [], foreign: ForeignEvent[] = [],
      includeOwner = false,
    ) => {
      for (const [id, delay] of calls) {
        const days = this.parseExactDelay(delay);
        if (days === null) continue;
        for (const target of this.timelineById.get(id.toLowerCase()) ?? []) {
          if (!allowed(target, owner)) continue;
          if (!target.broadcast && (includeOwner || (target.country_tag && target.country_tag !== owner))) {
            // A local call still belongs to its caller's scope, even when
            // the event is defined in another country's file.
            const key = `${owner}:${this.storyKey(target)}`;
            this.foreignStoryDays.set(key, Math.min(this.foreignStoryDays.get(key) ?? Infinity, day + days));
          }
          enqueue(target, owner, day + days);
        }
      }
      for (const { recipient, target, delay } of foreign) {
        const days = this.parseExactDelay(delay);
        if (days === null || recipient === owner) continue;
        for (const item of this.timelineById.get(target.toLowerCase()) ?? []) {
          if (!allowed(item, recipient)) continue;
          const key = `${recipient}:${this.storyKey(item)}`;
          this.foreignStoryDays.set(key, Math.min(this.foreignStoryDays.get(key) ?? Infinity, day + days));
          enqueue(item, recipient, day + days);
        }
      }
    };
    for (let index = 0; index < queue.length; index++) {
      const { item, tag, day } = queue[index];
      const effects = this.automaticEventCalls(item);
      routeEffects(tag, day, effects.local, effects.foreign);
      if (!item.broadcast) for (const { owner, focus } of this.completedFocusChain(tag, effects.completions)) {
        routeEffects(owner, day, focus.event_calls, focus.foreign_events, owner !== tag);
      }
      if (!item.broadcast) for (const completion of effects.completions) {
        this.indexForeignCompletion(tag, completion, day, false);
      }
    }
  }

  private completedFocusChain(owner: string, completions: FocusCompletion[]) {
    const queue = completions.map(({ id, tag, bypass }) => ({ id, tag: tag || owner, bypass }));
    const visited = new Set<string>();
    const result: { owner: string; focus: Chapter }[] = [];
    for (let index = 0; index < queue.length; index++) {
      const { id, tag, bypass } = queue[index];
      if (bypass) continue;
      const key = `${tag}:${id}`;
      if (visited.has(key)) continue;
      visited.add(key);
      const candidates = this.focusVariants.get(id)?.filter((chapter) => this.focusBelongsTo(chapter, tag)) ?? [];
      const focus = candidates.find((chapter) => chapter.initial_tree) ?? candidates[0];
      if (!focus) continue;
      result.push({ owner: tag, focus });
      for (const completion of focus.focus_completions ?? []) {
        queue.push({ id: completion.id, tag: completion.tag || tag, bypass: completion.bypass });
      }
    }
    return result;
  }

  private indexExternalDevelopments() {
    this.externalByTag.clear();
    const names = new Map(this.countries.map((country) => [country.tag, country.name]));
    const broadcasts: ExternalDevelopment[] = [];
    for (const focus of this.chapters) {
      const owner = focus.id.split('_')[0].toUpperCase();
      if (!names.has(owner)) continue;
      const byRecipient = new Map<string, ExternalDevelopment['events']>();
      for (const link of focus.foreign_events ?? []) {
        const delay = this.parseExactDelay(link.delay);
        if (delay === null) continue;
        for (const item of this.timelineById.get(link.target.toLowerCase()) ?? []) {
          if (!['event', 'news'].includes(item.kind)) continue;
          const events = byRecipient.get(link.recipient) ?? [];
          const duplicate = events.find((event) => event.item === item);
          if (duplicate) duplicate.delay = Math.min(duplicate.delay, delay);
          else events.push({ item, delay });
          byRecipient.set(link.recipient, events);
        }
      }
      const news = this.focusBroadcasts(focus);
      for (const [tag, events] of byRecipient) {
        const entries = this.externalByTag.get(tag) ?? [];
        const combined = [...events, ...news.filter(({ item }) => !events.some((entry) => entry.item === item))];
        entries.push({ focus, country: names.get(owner) || owner, events: combined });
        this.externalByTag.set(tag, entries);
      }
      if (news.some(({ item }) => item.super_event || item.options.some((option) => option.super_event))) {
        broadcasts.push({ focus, country: names.get(owner) || owner, events: news });
      }
    }
    for (const tag of names.keys()) {
      const entries = this.externalByTag.get(tag) ?? [];
      const existing = new Set(entries.map(({ focus }) => focus.id));
      for (const development of broadcasts) {
        if (!development.focus.id.startsWith(tag + '_') && !existing.has(development.focus.id)) {
          entries.push(development);
          existing.add(development.focus.id);
        }
      }
      this.externalByTag.set(tag, entries);
    }
  }

  private focusBroadcasts(focus: Chapter): ExternalDevelopment['events'] {
    const events = new Map<TimelineItem, number>();
    for (const [target, rawDelay] of focus.broadcast_calls ?? []) {
      const delay = this.parseExactDelay(rawDelay);
      if (delay === null) continue;
      for (const item of this.timelineById.get(target.toLowerCase()) ?? []) {
        if (item.broadcast) events.set(item, Math.min(events.get(item) ?? Infinity, delay));
      }
    }
    return [...events].map(([item, delay]) => ({ item, delay }));
  }

  async chooseMod(mod: ModInfo) {
    if (this.importingArchive) return;
    this.saved.confirmations = {};
    this.saved.wartime_starts = {};
    this.saved.route_history = [];
    this.saved.option_due_days = {};
    this.saved.option_estimated_days = {};
    this.saved.country_flag_changes = undefined;
    this.saved.story_choices = {};
    this.saved.active_focus_tree = undefined;
    this.saved.decision_started = {};
    this.saved.mission_started = {};
    this.saved.mission_deadline_adjustments = {};
    this.selectedMod = mod;
    this.saved.mod_path = mod.path;
    this.saved.tag = '';
    this.saved.focus_id = '';
    this.saved.last_kind = 'focus';
    this.saved.read_focus_ids = [];
    this.saved.excluded_focus_ids = [];
    this.saved.read_story_ids = [];
    this.saved.focus_completion_days = {};
    this.saved.route_day = 0;
    this.selectedCountry = null;
    this.current = null;
    this.countryQuery = '';
    this.showAllCountries = false;
    this.persist();
    this.setPage('countries');
    await this.scan();
  }

  chooseCountry(country: Country) {
    if (this.countryIndexPending) {
      this.pendingCountryTag = country.tag;
      return;
    }
    const changedCountry = this.saved.tag !== country.tag;
    this.selectedCountry = country;
    this.gameTextCache.clear();
    this.saved.tag = country.tag;
    if (this.selectedMod) {
      this.saved.last_mod_theme = this.themeForMod(
        this.selectedMod.workshop_id || this.selectedMod.path.split(/[\\/]/).pop() || '',
        this.selectedMod.name,
      );
      this.applyTheme();
    }
    this.saved.focus_id = '';
    this.saved.last_kind = 'focus';
    if (changedCountry) {
      this.saved.confirmations = {};
      this.saved.wartime_starts = {};
      this.saved.route_history = [];
      this.saved.option_due_days = {};
      this.saved.option_estimated_days = {};
      this.saved.country_flag_changes = undefined;
      this.saved.story_choices = {};
      this.saved.active_focus_tree = undefined;
      this.saved.decision_started = {};
      this.saved.mission_started = {};
      this.saved.mission_deadline_adjustments = {};
      this.saved.read_focus_ids = [];
      this.saved.excluded_focus_ids = [];
      this.saved.read_story_ids = [];
      this.saved.focus_completion_days = {};
      this.saved.route_day = 0;
    }
    this.saved.route_day = Math.max(this.saved.route_day || 0, this.openingWindowDays);
    this.current = null;
    this.persist();
    if (this.countryOpeningNotes.length) this.setPage('country-intro');
    else this.openRoutePage();
  }

  startCountryReading() {
    this.advanceAutomaticDecisions();
    this.schedulePeriodicStories();
    const opening = this.nextDueStory;
    if (opening) void this.readStory(opening);
    else this.openRoutePage();
  }

  get canUndoRoute() {
    return !!this.saved.route_history?.length;
  }

  private checkpointRoute(returnTo: RouteCheckpoint['returnTo']) {
    const {
      focus_id,
      last_kind,
      read_focus_ids,
      excluded_focus_ids,
      read_story_ids,
      route_day,
      focus_completion_days,
      last_title,
      last_country,
      last_mod,
    } = this.saved;
    const progress = {
      tag: this.saved.tag,
      focus_id,
      last_kind,
      read_focus_ids: [...read_focus_ids],
      excluded_focus_ids: [...excluded_focus_ids],
      read_story_ids: [...read_story_ids],
      route_day,
      focus_completion_days: { ...focus_completion_days },
      option_due_days: { ...this.saved.option_due_days },
      option_estimated_days: { ...this.saved.option_estimated_days },
      country_flag_changes: this.saved.country_flag_changes && { ...this.saved.country_flag_changes },
      story_choices: { ...this.saved.story_choices },
      active_focus_tree: this.saved.active_focus_tree,
      decision_started: { ...this.saved.decision_started },
      mission_started: { ...this.saved.mission_started },
      mission_deadline_adjustments: { ...this.saved.mission_deadline_adjustments },
      confirmations: { ...this.saved.confirmations },
      wartime_starts: { ...this.saved.wartime_starts },
      last_title,
      last_country,
      last_mod,
    };
    this.saved.route_history = [
      ...(this.saved.route_history ?? []).slice(-29),
      { returnTo, progress },
    ];
  }

  undoRoute() {
    const checkpoint = this.saved.route_history?.pop();
    if (!checkpoint) return;
    Object.assign(this.saved, checkpoint.progress);
    this.selectedCountry = this.countries.find(({ tag }) => tag === this.saved.tag) ?? this.selectedCountry;
    this.saved.option_due_days = { ...checkpoint.progress.option_due_days };
    this.saved.option_estimated_days = { ...checkpoint.progress.option_estimated_days };
    this.saved.country_flag_changes = checkpoint.progress.country_flag_changes &&
      { ...checkpoint.progress.country_flag_changes };
    this.saved.story_choices = { ...checkpoint.progress.story_choices };
    this.saved.active_focus_tree = checkpoint.progress.active_focus_tree;
    this.saved.decision_started = { ...checkpoint.progress.decision_started };
    this.saved.mission_started = { ...checkpoint.progress.mission_started };
    this.saved.mission_deadline_adjustments = { ...checkpoint.progress.mission_deadline_adjustments };
    this.saved.confirmations = { ...checkpoint.progress.confirmations };
    this.saved.wartime_starts = { ...checkpoint.progress.wartime_starts };
    this.restoreLegacyElectionChoice();
    this.restoreFocusTree();
    this.restoreTaiwanLoss();
    this.restoreEnvironmentalOutcome();
    this.persist();
    if (checkpoint.returnTo === 'event') {
      const story = this.saved.last_kind === 'focus'
        ? this.localFocus(this.saved.focus_id)
        : this.timelineByKey.get(`${this.saved.last_kind}:${this.saved.focus_id}`) ??
          this.saved.confirmations?.[`confirmation:${this.saved.focus_id}`]?.item;
      if (story) {
        this.showReader(story);
        return;
      }
    }
    this.current = null;
    this.selectedOption = null;
    this.branchMessage = '';
    this.openRoutePage();
  }

  previewStory(item: TimelineItem) {
    this.showReader(item);
    this.storyPreview = true;
  }

  get searchPreviewOptions() {
    return this.searchPreview && this.current && 'kind' in this.current ? this.current.options : [];
  }

  async openSearch() {
    this.setPage('search');
    if (await this.scan() && this.page === 'search') this.updateSearch();
    this.changeDetector.markForCheck();
  }

  private normalizeSearch(text: string) {
    return text.normalize('NFKC').toLowerCase().replace(/\s+/g, ' ').trim();
  }

  private openingSearchEntries(): TimelineItem[] {
    return Object.entries(this.openingNotes).flatMap(([tag, notes]) => notes
      .filter((note) => note.text)
      .map((note, index) => ({
        id: `opening:${tag}:${index}`, kind: 'opening-note', country_tag: tag,
        title: this.countryNames.get(tag) ?? tag, description: note.text, image_path: note.images[0] ?? null,
        source: this.t('开局提示'), date: null, timing: '', start_day: null, start_timing: null,
        next_ids: [], linked_from: [], options: [],
      })));
  }

  updateSearch(reset = true) {
    if (reset) this.searchLimit = 40;
    this.searchResults = [];
    this.searchCount = 0;
    const query = this.normalizeSearch(this.searchQuery);
    if (!query) return;
    const words = query.split(' ');
    this.searchDocuments ??= [...this.chapters, ...this.timeline.filter((item) => !item.hidden), ...this.openingSearchEntries()]
      .map((item) => {
        const field = (kind: SearchField['kind'], raw: string) => {
          const text = this.plainGameText(raw).replace(/\s+/g, ' ').trim();
          return { kind, raw, text, normalized: this.normalizeSearch(text) };
        };
        const fields = [field('title', item.title), field('description', item.description || '')];
        if ('kind' in item) {
          fields.push(...(item.title_variants ?? []).map(({ text }) => field('title', text)),
            ...(item.description_variants ?? []).map(({ text }) => field('description', text)),
            ...item.options.map(({ text }) => field('option', text)));
        }
        return { item, fields };
      });
    for (const { item, fields } of this.searchDocuments) {
      if (!words.every((word) => fields.some((field) => field.normalized.includes(word)))) continue;
      this.searchCount++;
      if (this.searchResults.length >= this.searchLimit) continue;
      const match = fields.slice(1).find((field) => words.some((word) => field.normalized.includes(word)));
      const excerpt = match ?? fields[1];
      const word = words.find((value) => excerpt.normalized.includes(value));
      const offset = word ? Math.max(0, excerpt.normalized.indexOf(word) - 32) : 0;
      const { text } = excerpt;
      this.searchResults.push({
        item,
        excerpt: (offset ? '…' : '') + text.slice(offset, offset + 156) + (text.length > offset + 156 ? '…' : ''),
        match,
      });
    }
  }

  showMoreSearchResults() {
    this.searchLimit += 40;
    this.updateSearch(false);
  }

  previewSearchResult(result: SearchResult) {
    this.showReader(result.item);
    this.storyPreview = true;
    this.searchPreview = true;
    this.searchMatch = result.match;
  }

  private showReader(item: Chapter | TimelineItem) {
    this.checkedDecisions.clear();
    this.wartimeChoices.clear();
    this.storyPreview = false;
    this.searchPreview = false;
    this.searchMatch = undefined;
    this.current = item;
    this.selectedOption = 'kind' in item
      ? item.options.find(({ id }) => id === this.saved.story_choices?.[this.storyKey(item)]) ?? null
      : null;
    this.branchMessage = '';
    this.image = this.imageSource(item.image_path);
    this.setPage('reader');
  }

  private restoreEnvironmentalOutcome() {
    if (this.saved.tag !== 'PRC') return;
    const environmental = new Set(['china.223', 'china.224', 'china.225']);
    const referenced = (route: ConditionalRoute): string[] => [
      ...(route.calls ?? []).map(([id]) => id), ...(route.foreign_events ?? []).map(({ target }) => target),
      ...(route.nested ?? []).flatMap((chain) => chain.flatMap(referenced)),
    ];
    const hasEvent = (id: string) => this.saved.read_story_ids.includes(`event:${id}`) ||
      this.saved.option_due_days?.[`event:${id}`] !== undefined;
    const severe = hasEvent('china.224') || hasEvent('china.225');
    if (this.saved.country_flag_changes?.['reader:PRC_environment_30'] === undefined && (severe || hasEvent('china.223'))) {
      this.applyFlagChanges([{ flag: 'reader:PRC_environment_30', present: true },
        { flag: 'reader:PRC_environment_20', present: severe }]);
    }
    for (const [key, entry] of Object.entries(this.saved.confirmations ?? {})) {
      if (key === 'confirmation:PRC_environment_outcome') continue;
      const targets = entry.item.confirmation?.choices.flatMap(({ route }) => route ? referenced(route) : []) ?? [];
      if (targets.length && targets.every((id) => environmental.has(id.toLowerCase()))) delete this.saved.confirmations?.[key];
    }
    if (this.saved.last_kind === 'confirmation' && !this.saved.confirmations?.[`confirmation:${this.saved.focus_id}`]) {
      this.saved.last_kind = 'focus';
      this.saved.focus_id = '';
    }
    this.persist();
  }

  async read(chapter: Chapter) {
    if (!this.saved.read_focus_ids.includes(chapter.id)) this.checkpointRoute('focus-choice');
    this.saved.focus_id = chapter.id;
    this.saved.last_kind = 'focus';
    this.saved.focus_completion_days ||= {};
    if (this.saved.focus_completion_days[chapter.id] === undefined) {
      const startDay = Math.max(this.saved.route_day || 0, this.openingWindowDays);
      this.scheduleScriptCalls(chapter.selection_calls, startDay);
      this.scheduleScopedCalls(chapter.selection_foreign_events, startDay);
      this.applyFlagChanges(chapter.selection_flags);
      this.playScriptedMusic(chapter.selection_music, this.saved.tag, startDay);
      this.activateMissions(chapter.selection_missions, startDay);
      this.applyFocusCompletions(chapter.selection_completions, startDay);
      this.applyConditionalRoutes(chapter.selection_routes, startDay, this.saved.tag, false, chapter, 'selection');
      this.saved.route_day =
        startDay + (chapter.completed_at_start ? 0 : chapter.duration_days || 70);
      this.saved.focus_completion_days[chapter.id] = this.saved.route_day;
      this.saved.read_focus_ids = [...new Set([...this.saved.read_focus_ids, chapter.id])];
      this.scheduleScriptCalls(chapter.event_calls, this.saved.route_day);
      this.scheduleScopedCalls(chapter.foreign_events, this.saved.route_day);
      for (const event of this.focusProgressStories.get(chapter.id) ?? []) {
        const due = this.focusProgressDay(event, startDay, this.saved.route_day);
        if (due !== null) this.scheduleEvent(event, '', due, true);
      }
      this.applyFlagChanges(chapter.flag_changes);
      this.playScriptedMusic(chapter.music, this.saved.tag, this.routeDay);
      this.applyTreeChanges(chapter.focus_tree_changes);
      this.applyFocusCompletions(chapter.focus_completions, this.saved.route_day);
      const deferred = this.dueStories.some((item) => (this.storyScheduleDay(item) ?? Infinity) < this.routeDay);
      if (deferred && chapter.conditional_routes?.length) {
        const day = this.saved.route_day;
        const id = `focus-completion:${chapter.id}`;
        const key = `confirmation:${id}`;
        (this.saved.confirmations ??= {})[key] = { day, item: {
          id, country_tag: this.saved.tag, kind: 'confirmation', title: chapter.title, description: '',
          date: null, source: chapter.source, timing: '', image_path: null, start_day: day, start_timing: null,
          linked_from: [], options: [], next_ids: [], focus_completion_routes: chapter.conditional_routes,
        } };
      } else {
        this.applyConditionalRoutes(chapter.conditional_routes, this.saved.route_day, this.saved.tag, false, chapter, 'completion');
      }
      this.activateMissions(chapter.mission_activations, this.routeDay);
      this.adjustMissionDeadlines(chapter.mission_time_changes);
    }
    this.saved.read_focus_ids = [...new Set([...(this.saved.read_focus_ids || []), chapter.id])];
    this.saved.excluded_focus_ids = [
      ...new Set([
        ...(this.saved.excluded_focus_ids || []),
        ...(chapter.mutually_exclusive_ids || []),
      ]),
    ];
    this.saved.last_title = chapter.title;
    this.scheduleClosingCredits(chapter, chapter.credits ?? [], this.routeDay);
    this.saved.last_country = this.selectedCountry?.name || this.saved.tag;
    this.saved.last_mod = this.selectedMod?.name || this.saved.mod_path;
    this.persist();
    this.showReader(chapter);
    this.openSuperEvent(chapter.super_event);
  }

  private advanceAutomaticDecisions() {
    this.applyAmericanFactionTransfer();
    this.applyProjectedFocusCompletions();
    this.advanceFocusBypasses();
    const tag = this.saved.tag;
    if (!tag) return;
    const warAlreadyStarted = this.saved.mission_started?.['PRC_GAW_war_exhaustion'] !== undefined ||
      this.saved.mission_started?.['PRC_korean_war_joining'] !== undefined;
    const candidates = this.countryTimeline.filter((item) => item.kind === 'decision' &&
      this.hasDecisionStories(item) && !this.saved.read_story_ids.includes(this.storyKey(item)));
    let advanced = false;
    // A decision can unlock another one by setting a flag.
    for (let changed = true; changed;) {
      changed = false;
      for (const item of candidates) {
        if (this.saved.read_story_ids.includes(this.storyKey(item))) continue;
        if (item.id === 'PRC_Liberals_Start_Gaw' &&
            (tag !== 'PRC' || !this.activeFocusTreeIds.has('china_liberal_focus') || warAlreadyStarted ||
            !item.decision_event_calls?.some(([id]) => id === 'china.183'))) continue;
        if (item.country_tag !== tag || !this.decisionAvailable(item) || !item.decision_condition) continue;
        const condition = evaluateCondition(item.decision_condition, (check) => this.flagMatches(check));
        const unlock = this.decisionUnlockDays(item);
        const focuses = (item.decision_focuses ?? []).map((id) => this.saved.focus_completion_days[id] ??
          (this.localFocus(id)?.completed_at_start ? 0 : this.routeDay));
        const decisionDay = Math.max(0, ...focuses, unlock.length ? Math.min(...unlock) : 0);
        if (condition !== true) {
          if (condition === null && (focuses.length || unlock.length) && item.id !== 'PRC_Liberals_Start_Gaw') {
            const key = `confirmation:${this.storyKey(item)}:available:${decisionDay}`;
            if (!this.saved.confirmations?.[key] && !this.saved.read_story_ids.includes(key)) {
              this.queueConfirmation(item, 'available', decisionDay, tag, false, [
                { decision: item.id, label: `执行「${item.title}」` }, { label: '条件未达成，继续阅读' },
              ]);
              advanced = true;
            }
          }
          continue;
        }
        if (this.saved.confirmations) delete this.saved.confirmations[`confirmation:${this.storyKey(item)}:available:${decisionDay}`];
        // Flag gates may have opened after the last focus completed. Without
        // a timestamp for the flag, use the day on which we observe it.
        const hasFlagGate = this.conditionUsesStoryFlag(item.decision_condition);
        const day = hasFlagGate || (!focuses.length && !unlock.length) ? this.routeDay : decisionDay;
        this.activateDecision(item, day);
        changed = advanced = true;
      }
    }
    if (advanced) this.persist();
  }

  private americanFactionTransfer() {
    if (this.saved.tag !== 'USA') return null;
    const selection = this.timelineByKey.get('event:usa.105')?.options.find(({ id }) =>
      id === this.saved.story_choices?.['event:usa.105']);
    const flag = selection?.flag_changes?.find(({ flag, present }) => present && /^global:play_[A-Z0-9]{3}_global$/.test(flag));
    const tag = flag?.flag.slice('global:play_'.length, -'_global'.length);
    const country = tag && this.countries.find((country) => country.tag === tag);
    const split = this.timelineByKey.get('event:usa.6');
    const transfer = tag === 'USC' ? this.timelineByKey.get('event:usa.15') : split;
    const day = split && this.storyScheduleDay(split);
    const call = split?.immediate_calls?.find(([id]) => id === transfer?.id);
    const delay = split === transfer ? 0 : call ? this.parseStoryDelay(call[1]) : null;
    return country && split && transfer && day != null && delay !== null ? { country, split, transfer, day: day + delay } : null;
  }

  private applyAmericanFactionTransfer() {
    const change = this.americanFactionTransfer();
    if (!change || change.day > this.routeDay || this.dueStories.some((story) =>
      (this.storyScheduleDay(story) ?? Infinity) < change.day)) return;
    this.saved.tag = change.country.tag;
    this.saved.last_country = change.country.name;
    this.selectedCountry = change.country;
    this.saved.active_focus_tree = undefined;
    const pending = this.saved.option_due_days ?? {};
    for (const key of Object.keys(pending)) {
      const item = this.timelineByKey.get(key);
      if (item?.country_tag === 'USA' && !item.broadcast && pending[key] > change.day) {
        delete pending[key];
        if (this.saved.option_estimated_days) delete this.saved.option_estimated_days[key];
      }
    }
    for (const item of new Set([change.split, change.transfer])) {
      this.scheduleScopedCalls(item.foreign_events, change.day);
    }
    this.persist();
  }

  private periodicStoriesByOwner = new Map<string, TimelineItem[]>();
  private focusConditionFlags = new Set<string>();
  private autonomousStoriesByOwner = new Map<string, TimelineItem[]>();
  private manualTriggersByOwner = new Map<string, TimelineItem[]>();

  private scheduleAutonomousStories() {
    const pending = this.saved.option_due_days ??= {};
    const estimates = this.saved.option_estimated_days ??= {};
    const read = new Set(this.saved.read_story_ids);
    let changed = false;
    for (const item of this.autonomousStoriesByOwner.get(this.saved.tag) ?? []) {
      const trigger = item.autonomous_trigger;
      const key = this.storyKey(item);
      if (!trigger || read.has(key)) continue;
      const eligible = evaluateCondition(trigger.condition, (check) => this.flagMatches(check)) === true;
      if (!eligible) {
        // The event's condition must still hold when a pending random event fires.
        if (pending[key] !== undefined) {
          delete pending[key];
          delete estimates[key];
          changed = true;
        }
        continue;
      }
      if (pending[key] !== undefined) continue;
      const day = this.routeDay;
      pending[key] = day + (trigger.mean_days ?? 0);
      if (trigger.mean_days !== null) estimates[key] = pending[key];
      changed = true;
    }
    if (changed) this.persist();
  }

  private scheduleAnchoredConfirmations() {
    const tag = this.saved.tag;
    this.scheduleFrenchCivilWarOutcome();
    for (const item of this.manualTriggersByOwner.get(tag) ?? []) {
      const trigger = item.manual_trigger!;
      const after = trigger.earliest ? this.calendarDay(trigger.earliest) : 0;
      const before = trigger.latest ? this.calendarDay(trigger.latest) : Infinity;
      if (after === null || before === null || this.saved.read_story_ids.includes(this.storyKey(item))) continue;
      const focuses = trigger.focuses.map((id) => this.saved.focus_completion_days[id] ??
        (this.localFocus(id)?.completed_at_start ? 0 : Infinity));
      const day = Math.max(after + (trigger.earliest && !trigger.earliest_inclusive ? 1 : 0), ...focuses);
      if (!Number.isFinite(day) || day > before || (day === before && !trigger.latest_inclusive)) continue;
      const key = `confirmation:${this.storyKey(item)}:trigger:${day}`;
      const saved = this.saved.confirmations;
      const confirmation = saved?.[key];
      if (this.saved.read_story_ids.includes(key) || this.storyScheduleDay(item) !== null) continue;
      // Known flags can change before a dated event is due.
      const matches = day > this.routeDay ? null : this.evaluateForCountry(trigger.condition, tag, day);
      if (matches !== null && confirmation && saved) {
        delete saved[key];
        this.persist();
      }
      if (matches === false) continue;
      if (matches === true) this.scheduleEvent(item, '', day);
      else if (!confirmation) this.queueConfirmation(item, 'trigger', day, tag, false, [
        { event: item.id, label: `发生「${item.title}」` }, { label: '条件未达成，继续阅读' },
      ]);
      if (matches === true || !confirmation) this.persist();
    }
  }

  private scheduleFrenchCivilWarOutcome() {
    const flag = 'global:French_Civil_War_Global_Flag';
    const finishers: Record<string, string> = { FRA: 'france.371', FPR: 'france.372', FAF: 'france.373' };
    const localWinner = finishers[this.saved.tag];
    const opening = ['france.37', 'france.374'].flatMap((id) => {
      const story = this.timelineByKey.get(`event:${id}`);
      return story && this.saved.read_story_ids.includes(this.storyKey(story)) &&
          story.flag_changes?.some((change) => change.flag === flag && change.present) ? [story] : [];
    })[0];
    if (!localWinner || !opening || !this.saved.country_flag_changes?.[flag] || this.availableFocuses.length) return;
    const endings = Object.values(finishers).flatMap((id) => {
      const story = this.timelineByKey.get(`event:${id}`);
      return story?.flag_changes?.some((change) => change.flag === flag && !change.present) ? [story] : [];
    });
    if (endings.length !== 3) return;
    const day = Math.max(this.storyScheduleDay(opening) ?? this.routeDay,
      ...Object.values(this.saved.focus_completion_days));
    this.queueConfirmation(opening, 'french-civil-war', day, this.saved.tag, true,
      endings.map((story) => {
        const winner = Object.keys(finishers).find((tag) => finishers[tag] === story.id);
        return { label: this.countries.find(({ tag }) => tag === winner)?.name ?? story.title, event: story.id };
      }), {
        id: 'french-civil-war-outcome', title: '法国内战的结局', description: '请选择控制法国本土的阵营。',
      });
  }

  private schedulePeriodicStories() {
    this.scheduleAutonomousStories();
    this.scheduleAnchoredConfirmations();
    const tag = this.saved.tag;
    const day = this.routeDay;
    const pending = this.saved.option_due_days ??= {};
    const estimates = this.saved.option_estimated_days ??= {};
    const read = new Set(this.saved.read_story_ids);
    let changed = false;
    for (const item of this.periodicStoriesByOwner.get(tag) ?? []) {
      const key = this.storyKey(item);
      if (read.has(key) || pending[key] !== undefined || !this.checksPass(item.trigger_checks ?? [])) continue;
      for (const trigger of item.periodic_triggers ?? []) {
        if (trigger.owner !== tag || (trigger.recipient !== tag && !item.broadcast) ||
            (item.startup_trigger && item.startup_trigger !== trigger.recipient) ||
            evaluateCondition(trigger.condition, (check) => this.flagMatches(check)) !== true) continue;
        const delay = this.parseStoryDelay(trigger.delay);
        if (delay === null) continue;
        // Use the next check in the reader's calendar. The engine's weekly
        // polling phase is not recorded in the mod files.
        const due = Math.ceil((day + 1) / trigger.interval) * trigger.interval + delay;
        if (due >= (pending[key] ?? Infinity)) continue;
        estimates[key] = pending[key] = due;
        changed = true;
      }
    }
    if (changed) this.persist();
  }

  private conditionUsesStoryFlag(condition: ScriptCondition<CountryFlag>): boolean {
    if (condition.op === 'check') {
      const flag = condition.args.flag;
      return !flag.startsWith('focus:') && !flag.startsWith('tag:') &&
        !flag.startsWith('rule:') && flag !== 'is_ai';
    }
    if (condition.op === 'not') return this.conditionUsesStoryFlag(condition.args);
    return (condition.op === 'all' || condition.op === 'any') &&
      condition.args.some((term) => this.conditionUsesStoryFlag(term));
  }

  private activateDecision(item: TimelineItem, day = this.routeDay) {
    const key = this.storyKey(item);
    if (!this.decisionAvailable(item) || this.saved.read_story_ids.includes(key)) return;
    const calls = [...(item.decision_event_calls ?? [])];
    for (const call of item.decision_guarded_calls ?? []) {
      if (!call.deferred && this.checksPass(call.checks)) calls.push([call.target, call.delay]);
    }
    this.scheduleScriptCalls(calls, day);
    this.scheduleScopedCalls(item.foreign_events, day);
    this.saved.decision_started = { ...this.saved.decision_started, [item.id]: day };
    this.applyFlagChanges(item.flag_changes);
    this.playScriptedMusic(item.music, this.saved.tag, day);
    this.applyTreeChanges(item.focus_tree_changes);
    this.activateStoryMissions(item, day);
    this.adjustMissionDeadlines(item.mission_time_changes);
    this.applyConditionalRoutes(item.conditional_routes, day, this.saved.tag, false, item);
    this.applyFocusCompletions(item.focus_completions, day);
    this.saved.read_story_ids = [...this.saved.read_story_ids, key];
    return item.super_event;
  }

  chooseDecision(item: TimelineItem) {
    if (!this.decisionAvailable(item) || this.saved.read_story_ids.includes(this.storyKey(item))) return;
    this.checkpointRoute('focus-choice');
    const scene = this.activateDecision(item);
    this.persist();
    this.openSuperEvent(scene, () => {
      const due = this.nextDueStory;
      if (due) void this.readStory(due);
    });
  }

  async readStory(item: TimelineItem) {
    if (this.hasEnding(item) && this.shouldDeferEnding()) {
      this.openRoutePage();
      return;
    }
    if (item.focus_completion_routes) {
      const day = this.storyScheduleDay(item);
      const key = this.storyKey(item);
      if (day === null || this.saved.read_story_ids.includes(key)) return;
      this.saved.read_story_ids = [...this.saved.read_story_ids, key];
      this.applyConditionalRoutes(item.focus_completion_routes, day, item.country_tag, false,
        this.localFocus(item.id.slice('focus-completion:'.length)) ?? item, 'completion');
      this.persist();
      await this.continueRoute();
      return;
    }
    if (item.kind === 'decision-end' && item.decision_end_music) {
      const day = this.storyScheduleDay(item);
      if (day === null || day > this.routeDay || this.saved.read_story_ids.includes(this.storyKey(item))) return;
      const ending = item.decision_end_music;
      this.saved.read_story_ids = [...this.saved.read_story_ids, this.storyKey(item)];
      this.applyFlagChanges(ending.flags);
      this.playScriptedMusic(ending.effects, item.country_tag || this.saved.tag, day);
      this.applyConditionalRoutes(ending.conditional_routes, day, item.country_tag || this.saved.tag);
      this.persist();
      await this.continueRoute();
      return;
    }
    if (item.kind === 'decision') {
      this.chooseDecision(item);
      return;
    }
    if (item.kind === 'mission' && this.automaticallyTimesOut(item) && item.mission) {
      const day = this.storyScheduleDay(item);
      if (day === null || day > this.routeDay) return;
      const effect = item.mission.timeout;
      this.saved.read_story_ids = [...this.saved.read_story_ids, this.storyKey(item)];
      this.applyFlagChanges(effect.flags);
      this.playScriptedMusic(effect.music, item.country_tag || this.saved.tag, day);
      this.applyTreeChanges(effect.trees);
      this.applyFocusCompletions(effect.completions, day);
      this.scheduleScriptCalls(effect.calls, day);
      this.scheduleScopedCalls(effect.foreign_events, day);
      this.activateMissions(effect.activated_missions?.filter(([owner]) => owner === this.saved.tag).map(([, id]) => id), day);
      this.applyConditionalRoutes(effect.conditional_routes, day, this.saved.tag, false, item, 'timeout');
      this.persist();
      await this.continueRoute();
      return;
    }
    if (item.hidden) {
      const day = this.storyScheduleDay(item);
      const key = this.storyKey(item);
      if (day === null || day > this.routeDay || this.saved.read_story_ids.includes(key)) return;
      this.saved.read_story_ids = [...this.saved.read_story_ids, key];
      this.applyFlagChanges(item.flag_changes);
      this.applyTreeChanges(item.focus_tree_changes);
      this.applyFocusCompletions(item.focus_completions, day);
      this.activateStoryMissions(item, day);
      this.adjustMissionDeadlines(item.mission_time_changes);
      this.playScriptedMusic(item.music, item.country_tag || this.saved.tag, day);
      this.applyConditionalRoutes(item.conditional_routes, day, this.saved.tag, this.isProjectedStory(item), item);
      this.persist();
      this.openSuperEvent(item.super_event, () => void this.continueRoute());
      return;
    }
    this.current = item;
    this.saved.focus_id = item.id;
    this.saved.last_kind = item.kind;
    if (item.kind === 'mission' || item.confirmation) {
      this.saved.last_title = item.title;
      this.persist();
      this.showReader(item);
      return;
    }
    if (!this.saved.read_story_ids.includes(this.storyKey(item))) {
      this.applyFlagChanges(item.flag_changes);
      this.applyTreeChanges(item.focus_tree_changes);
      const day = this.storyScheduleDay(item) ?? this.routeDay;
      this.playScriptedMusic(item.music, item.broadcast ? this.saved.tag : item.country_tag || this.saved.tag, day);
      this.activateStoryMissions(item, day);
      this.adjustMissionDeadlines(item.mission_time_changes);
      this.applyConditionalRoutes(item.conditional_routes, day, this.saved.tag, this.isProjectedStory(item), item, 'immediate');
      if (item.flag_changes?.some(({ flag, present }) => flag === 'global:USA_american_civil_war_over_global' && present)) {
        this.queueWartimeDevelopments('USA_american_civil_war_global', item, day, true);
      }
    }
    this.applyFocusCompletions(item.focus_completions, this.storyScheduleDay(item) ?? this.routeDay);
    this.scheduleClosingCredits(item, this.japaneseDefeatCredits(item), this.storyScheduleDay(item) ?? this.routeDay);
    this.saved.read_story_ids = [...new Set([...this.saved.read_story_ids, this.storyKey(item)])];
    this.saved.last_title = this.storyText(item.title_variants) ?? item.title;
    this.saved.last_country = this.selectedCountry?.name || this.saved.tag;
    this.saved.last_mod = this.selectedMod?.name || this.saved.mod_path;
    this.persist();
    this.showReader(item);
    this.openSuperEvent(item.super_event);
  }

  async chooseStoryOption(option: StoryOption, recordChoice = this.hasStoryBranches) {
    if (this.storyPreview || this.selectedOption || !this.storyOptions.includes(option)) return;
    if (recordChoice || option.focus_tree_changes?.length) this.checkpointRoute('event');
    const current = this.current;
    const eventDay = current && 'kind' in current ? this.storyScheduleDay(current) : null;
    const projected = !!(current && 'kind' in current && this.isProjectedStory(current));
    if (current && 'kind' in current) {
      this.saved.story_choices = { ...this.saved.story_choices, [this.storyKey(current)]: option.id };
    }
    this.applyFlagChanges(option.flag_changes);
    this.playScriptedMusic(option.music, this.saved.tag, eventDay ?? this.routeDay);
    this.applyTreeChanges(option.focus_tree_changes);
    this.applyFocusCompletions(option.focus_completions, eventDay ?? this.routeDay);
    this.activateMissions(option.mission_activations, eventDay ?? this.routeDay);
    this.adjustMissionDeadlines(option.mission_time_changes);
    this.applyConditionalRoutes(option.conditional_routes, eventDay ?? this.routeDay, this.saved.tag, projected, current ?? undefined, `option:${option.id}`);
    this.selectedOption = option;
    this.scheduleScriptCalls(option.event_calls?.filter(([id]) =>
      this.optionAppliesToCountry(option, id) && id.toLowerCase() !== current?.id.toLowerCase()), eventDay ?? this.routeDay, projected);
    this.scheduleScopedCalls(option.foreign_events?.filter(({ target }) =>
      target.toLowerCase() !== current?.id.toLowerCase()), eventDay ?? this.routeDay, projected);
    if (current && 'kind' in current && option.flag_changes?.some(({ flag, present }) =>
      flag === 'global:USA_american_civil_war_over_global' && present)) {
      this.queueWartimeDevelopments('USA_american_civil_war_global', current, eventDay ?? this.routeDay, true);
    }
    this.persist();

    if (this.readerDecisions.length) {
      this.openSuperEvent(option.super_event);
      return;
    }
    const unresolved = this.optionDestinations;
    if (unresolved.length > 1) {
      this.branchMessage = '这些后续事件的触发条件尚未确定，可以按剧情线索查看。';
      this.openSuperEvent(option.super_event);
    } else if (unresolved.length === 1 && !this.nextDueStory && !this.currentCredits.length) {
      this.openSuperEvent(option.super_event, () => this.followStory(unresolved[0]));
    } else {
      this.branchMessage = '';
      this.openSuperEvent(option.super_event, () => void this.continueRoute());
    }
  }

  followStory(item: TimelineItem) {
    if (this.hasEnding(item) && this.shouldDeferEnding()) {
      this.openRoutePage();
      return;
    }
    const day = this.storyScheduleDay(item);
    if (day !== null && day > this.routeDay) {
      this.saved.route_day = day;
      this.persist();
    }
    if (item.kind === 'confirmation') {
      this.advanceAutomaticDecisions();
      this.scheduleAnchoredConfirmations();
      const due = this.nextDueStory;
      if (due) void this.readStory(due);
      else this.openRoutePage();
      return;
    }
    void this.readStory(item);
  }

  async resumeRead() {
    if (!this.selectedMod) {
      this.setPage('mods');
      return;
    }
    if (!(await this.scan())) return;
    this.advanceAutomaticDecisions();
    this.schedulePeriodicStories();
    if (this.saved.last_kind && this.saved.last_kind !== 'focus') {
      const story = this.saved.confirmations?.[`confirmation:${this.saved.focus_id}`]?.item ?? this.timeline.find(
        (item) => item.id === this.saved.focus_id && item.kind === this.saved.last_kind,
      );
      if (story && (!this.hasEnding(story) || !this.shouldDeferEnding())) {
        this.showReader(story);
        return;
      }
    }
    const chapter = this.localFocus(this.saved.focus_id);
    if (chapter) this.showReader(chapter);
    else if (this.countryNames.has(this.saved.tag)) this.openRoutePage();
    else this.setPage('countries');
  }

  async browse(which: 'game_path' | 'workshop_path' | 'mod_path') {
    const path = await open({ directory: true, multiple: false });
    if (typeof path !== 'string') return;
    if (which === 'mod_path' && path !== this.saved.mod_path) {
      this.saved.confirmations = {};
      this.saved.wartime_starts = {};
      this.saved.route_history = [];
      this.saved.option_due_days = {};
      this.saved.option_estimated_days = {};
      this.saved.country_flag_changes = undefined;
      this.saved.story_choices = {};
      this.saved.active_focus_tree = undefined;
      this.saved.decision_started = {};
      this.saved.mission_started = {};
      this.saved.mission_deadline_adjustments = {};
    }
    this.saved[which] = path;
    if (which === 'workshop_path') {
      this.mods = await invoke<ModInfo[]>('scan_workshop_dir', { root: path });
    }
    if (which === 'mod_path') {
      this.selectedMod = {
        name: path.split(/[\\/]/).pop() || 'Mod',
        path,
        workshop_id: '',
        image_path: null,
      };
      this.setPage('countries');
      await this.scan();
    }
    this.persist();
  }

  async importArchive() {
    if (this.importingArchive) return;
    this.importingArchive = true;
    this.error = '';
    try {
      const uri = await open({ multiple: false, filters: [{ name: 'ZIP', extensions: ['zip'] }] });
      if (typeof uri !== 'string') return;
      const library = await invoke<Discovery>('import_archive', { uri });
      this.applyImportedLibrary(library);
    } catch (error) {
      this.error = String(error);
    } finally {
      this.importingArchive = false;
      this.changeDetector.markForCheck();
    }
  }

  async downloadArchive() {
    if (!this.android || this.importingArchive || !this.downloadUrl.trim()) return;
    this.error = '';
    this.importingArchive = true;
    this.downloadProgress = { received: 0, total: null, installing: false };
    const onProgress = new Channel<DownloadProgress>((progress) => {
      if (!this.importingArchive) return;
      this.downloadProgress = progress;
      this.changeDetector.markForCheck();
    });
    try {
      this.applyImportedLibrary(await invoke<Discovery>('download_archive', { url: this.downloadUrl.trim(), onProgress }));
      this.downloadUrl = '';
      this.showDownloadForm = false;
    } catch (error) {
      this.error = String(error);
    } finally {
      this.importingArchive = false;
      this.downloadProgress = null;
      this.changeDetector.markForCheck();
    }
  }

  private applyImportedLibrary(library: Discovery) {
    this.saved.game_path = library.game_path;
    this.saved.workshop_path = library.workshop_path;
    this.mods = library.mods;
    this.loadedScanKey = null;
    this.searchDocuments = null;
    this.tfrMenuArtPath = '';
    this.persist();
    void this.loadTfrMenuArt();
  }

  async rescan() {
    if (this.importingArchive) return;
    if (this.saved.workshop_path) {
      this.mods = await invoke<ModInfo[]>('scan_workshop_dir', {
        root: this.saved.workshop_path,
      });
    } else {
      await this.refreshMods();
    }
    if (this.selectedMod) await this.scan(true);
  }

  setPage(page: Page) {
    this.effectTooltip = null;
    this.closeOptionEffects();
    this.changeDetector.markForCheck();
    if (page !== 'countries') this.pendingCountryTag = null;
    if (page === 'home') void this.loadTfrMenuArt();
    if (page !== this.page) this.storyArchiveOpen = false;
    this.page = page;
    if (this.android) this.updateAndroidHistory();
    if (page === 'countries' && this.showAllCountries) void this.loadSecondaryCountries();
    queueMicrotask(() => {
      const content = document.querySelector<HTMLElement>('.content');
      if (content) {
        content.scrollTop = 0;
        content.scrollLeft = 0;
      }
    });
  }

  private updateAndroidHistory() {
    const guarded = history.state?.hoi4Reader;
    if (this.page !== 'home' && !guarded) history.pushState({ hoi4Reader: true }, '');
    if (this.page === 'home' && guarded) history.back();
  }

  @HostListener('window:popstate')
  onAndroidBack() {
    if (!this.android || this.page === 'home') return;
    if (this.optionEffectsSheet) {
      this.closeOptionEffects();
      this.updateAndroidHistory();
    } else if (this.superEvent) {
      this.closeSuperEvent();
      this.updateAndroidHistory();
    } else {
      this.back();
    }
  }

  @HostListener('window:keydown', ['$event'])
  handleFocusPickerShortcut(event: KeyboardEvent) {
    if (this.page !== 'focus-choice' || this.superEvent) return;
    if (event.key === 'Escape') this.back();
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'z' && this.canUndoRoute) {
      event.preventDefault();
      this.undoRoute();
    }
  }

  back() {
    if (this.page === 'reader' && this.searchPreview) {
      this.searchPreview = false;
      this.searchMatch = undefined;
      this.storyPreview = false;
      this.current = null;
      this.setPage('search');
    }
    else if (this.page === 'reader') this.openRoutePage();
    else if (this.page === 'focus-choice') this.setPage('countries');
    else if (this.page === 'chapters' || this.page === 'country-intro') this.setPage('countries');
    else if (this.page === 'countries') this.setPage('mods');
    else this.setPage('home');
  }
}
