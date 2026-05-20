"""add owner_id to projects and reference_files

Revision ID: b0f1f6a9c321
Revises: 416cd372ad39
Create Date: 2026-05-20
"""
from alembic import op
import sqlalchemy as sa

revision = 'b0f1f6a9c321'
down_revision = '416cd372ad39'
branch_labels = None
depends_on = None


def upgrade():
    op.add_column('projects', sa.Column('owner_id', sa.String(length=64), nullable=True))
    op.add_column('reference_files', sa.Column('owner_id', sa.String(length=64), nullable=True))
    op.create_index('ix_projects_owner_id', 'projects', ['owner_id'], unique=False)
    op.create_index('ix_reference_files_owner_id', 'reference_files', ['owner_id'], unique=False)


def downgrade():
    op.drop_index('ix_reference_files_owner_id', table_name='reference_files')
    op.drop_index('ix_projects_owner_id', table_name='projects')
    op.drop_column('reference_files', 'owner_id')
    op.drop_column('projects', 'owner_id')

