export interface ExampleRecord {
  id: string;
  entityType: 'model' | 'bundle';
  entityId: string;
  exampleType: 'image' | 'prompt';
  content: string;
  createdAt: string;
}
