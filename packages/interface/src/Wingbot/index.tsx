// Wingbot Router-based exports
export {WingbotProvider, useWingbot} from './WingbotContext';
export {WingbotLayout} from './WingbotLayout';
export {wingbotRoutes, WingbotRouter} from './router';

// Route components
export {ChatRoute} from './routes/ChatRoute';
export {ConversationRoute} from './routes/ConversationRoute';
export {TasksRoute} from './routes/TasksRoute';
export {MemoriesRoute} from './routes/MemoriesRoute';
export {AutonomyRoute} from './routes/AutonomyRoute';
export {ScheduleRoute} from './routes/ScheduleRoute';

// Reusable components
export {ChatComposer} from './ChatComposer';
export {ConversationScreen} from './ConversationScreen';
export {useWingbotEventSource} from './useWingbotEventSource';

// Export types
export type {WingbotContextType} from './WingbotContext';

